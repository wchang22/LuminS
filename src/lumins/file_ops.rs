use std::marker::Sync;
use std::path::PathBuf;
use std::{fs, io};

use blake2::{Blake2b, Digest};
use rayon::prelude::*;
use rayon_hash::HashSet;
use seahash::hash;

use crate::lumins::parse;

/// Interface for all file structs to perform common operations
///
/// Ensures that all files (file, dir, symlink) have
/// a way of obtaining their path, copying, and deleting
pub trait FileOps {
    fn path(&self) -> &PathBuf;
    fn remove(&self, path: &PathBuf);
    fn copy(&self, src: &PathBuf, dest: &PathBuf);
}

/// A struct that represents a single file
#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct File {
    path: PathBuf,
    size: u64,
}

impl FileOps for File {
    fn path(&self) -> &PathBuf {
        &self.path
    }
    fn remove(&self, path: &PathBuf) {
        let remove = fs::remove_file(&path);
        if remove.is_err() {
            eprintln!(
                "Error -- Deleting File {:?}: {}",
                path,
                remove.err().unwrap()
            );
        } else {
            info!("Deleting {:?}", path);
        }
    }
    fn copy(&self, src: &PathBuf, dest: &PathBuf) {
        let copy = fs::copy(&src, &dest);
        if copy.is_err() {
            eprintln!("Error -- Copying {:?} {}", src, copy.err().unwrap());
        } else {
            info!("Copying {:?}", src);
        }
    }
}

/// A struct that represents a single directory
#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct Dir {
    path: PathBuf,
}

impl FileOps for Dir {
    fn path(&self) -> &PathBuf {
        &self.path
    }
    fn remove(&self, path: &PathBuf) {
        let remove = fs::remove_dir(&path);
        if remove.is_err() {
            eprintln!(
                "Error -- Deleting Dir {:?}: {}",
                path,
                remove.err().unwrap()
            );
        } else {
            info!("Deleting {:?}", path);
        }
    }
    fn copy(&self, _src: &PathBuf, dest: &PathBuf) {
        let create_dir = fs::create_dir_all(&dest);
        if create_dir.is_err() {
            eprintln!(
                "Error -- Creating directory {:?} {}",
                dest,
                create_dir.err().unwrap()
            );
        } else {
            info!("Creating {:?}", dest);
        }
    }
}

/// A struct that represents a single symbolic link
#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct Symlink {
    path: PathBuf,
    target: PathBuf,
}

impl FileOps for Symlink {
    fn path(&self) -> &PathBuf {
        &self.path
    }
    fn remove(&self, path: &PathBuf) {
        let remove = fs::remove_file(&path);
        if remove.is_err() {
            eprintln!(
                "Error -- Deleting Symlink {:?}: {}",
                path,
                remove.err().unwrap()
            );
        } else {
            info!("Deleting {:?}", path);
        }
    }
    #[cfg(target_family = "unix")]
    fn copy(&self, src: &PathBuf, dest: &PathBuf) {
        use std::os::unix::fs::symlink;

        let copy = symlink(&self.target, &dest);
        if copy.is_err() {
            eprintln!("Error -- Copying {:?} {}", src, copy.err().unwrap());
        } else {
            info!("Creating {:?}", dest);
        }
    }
}

/// A struct that represents sets of different types of files
#[derive(Eq, PartialEq, Debug)]
pub struct FileSets {
    files: HashSet<File>,
    dirs: HashSet<Dir>,
    symlinks: HashSet<Symlink>,
}

impl FileSets {
    /// Initializes FileSets with the given sets
    ///
    /// # Arguments
    /// * `files`: a set of files
    /// * `dirs`: a set of dirs
    /// * `symlinks`: a set of symlinks
    ///
    /// # Returns
    /// A newly created FileSets struct
    pub fn with(files: HashSet<File>, dirs: HashSet<Dir>, symlinks: HashSet<Symlink>) -> Self {
        FileSets {
            files,
            dirs,
            symlinks,
        }
    }
    /// Gets the set of files
    ///
    /// # Returns
    /// The FileSets set of files
    pub fn files(&self) -> &HashSet<File> {
        &self.files
    }
    /// Gets the set of dirs
    ///
    /// # Returns
    /// The FileSets set of dirs
    pub fn dirs(&self) -> &HashSet<Dir> {
        &self.dirs
    }
    /// Gets the set of symlinks
    ///
    /// # Returns
    /// The FileSets set of symlinks
    pub fn symlinks(&self) -> &HashSet<Symlink> {
        &self.symlinks
    }
}

/// Compares all files in `files_to_compare` in `src` with all files in `files_to_compare` in `dest`
/// and copies them over if they are different
///
/// # Arguments
/// * `files_to_compare`: files to compare
/// * `src`: base directory of the files to copy from, such that for all `file` in
/// `files_to_compare`, `src + file.path()` is the absolute path of the source file
/// * `dest`: base directory of the files to copy to, such that for all `file` in
/// `files_to_compare`, `dest + file.path()` is the absolute path of the destination file
/// * `flags`: bitfield for flags
pub fn compare_and_copy_files<'a, T, S>(files_to_compare: T, src: &str, dest: &str, flags: u32)
where
    T: ParallelIterator<Item = &'a S>,
    S: FileOps + Sync + 'a,
{
    files_to_compare.for_each(|file| {
        let secure = parse::contains_flag(flags, parse::Flag::Secure);

        let mut src_file_hash_secure = None;
        let mut src_file_hash = None;
        if secure {
            src_file_hash_secure = hash_file_secure(file, &src);
        } else {
            src_file_hash = hash_file(file, &src);
        }

        if (secure && src_file_hash_secure.is_none()) || (!secure && src_file_hash.is_none()) {
            copy_file(file, &src, &dest);
            return;
        }

        let mut dest_file_hash_secure = None;
        let mut dest_file_hash = None;
        if secure {
            dest_file_hash_secure = hash_file_secure(file, &dest);
        } else {
            dest_file_hash = hash_file(file, &dest);
        }

        if (secure && src_file_hash_secure != dest_file_hash_secure)
            || (!secure && src_file_hash != dest_file_hash)
        {
            copy_file(file, &src, &dest);
        }
    });
}

/// Copies all given files from `src` to `dest` in parallel
///
/// # Arguments
/// * `files_to_copy`: files to copy
/// * `src`: base directory of the files to copy from, such that for all `file` in
/// `files_to_copy`, `src + file.path()` is the absolute path of the source file
/// * `dest`: base directory of the files to copy to, such that for all `file` in
/// `files_to_copy`, `dest + file.path()` is the absolute path of the destination file
pub fn copy_files<'a, T, S>(files_to_copy: T, src: &str, dest: &str)
where
    T: ParallelIterator<Item = &'a S>,
    S: FileOps + Sync + 'a,
{
    files_to_copy.for_each(|file| {
        copy_file(file, &src, &dest);
    });
}

/// Copies a single file from `src` to `dest`
///
/// # Arguments
/// * `files_to_copy`: file to copy
/// * `src`: base directory of the files to copy from, such that `src + file_to_copy.path()`
/// is the absolute path of the source file
/// * `dest`: base directory of the files to copy to, such that `dest + file.path()`
/// is the absolute path of the destination file
fn copy_file<S>(file_to_copy: &S, src: &str, dest: &str)
where
    S: FileOps,
{
    let mut src_file = PathBuf::from(&src);
    src_file.push(&file_to_copy.path());
    let mut dest_file = PathBuf::from(&dest);
    dest_file.push(&file_to_copy.path());

    file_to_copy.copy(&src_file, &dest_file);
}

/// Deletes all given files in parallel
///
/// There is no guarantee that this function will delete the files in the given order
///
/// # Arguments
/// `files_to_delete`: files to delete
/// * `location`: base directory of the files to delete, such that for all `file` in
/// `files_to_delete`, `location + file.path()` is the absolute path of the file
pub fn delete_files<'a, T, S>(files_to_delete: T, location: &str)
where
    T: ParallelIterator<Item = &'a S>,
    S: FileOps + Sync + 'a,
{
    files_to_delete.for_each(|file| {
        let mut path = PathBuf::from(&location);
        path.push(file.path());

        file.remove(&path);
    });
}

/// Deletes all given files sequentially
///
/// This function ensures that the files are deleted in the exact order given
///
/// # Arguments
/// * `files_to_delete`: files to delete, or sorted empty directories
/// * `location`: base directory of the files to delete, such that for all `file` in
/// `files_to_delete`, `location + file.path()` is the absolute path of the file
pub fn delete_files_sequential<'a, T, S>(files_to_delete: T, location: &str)
where
    T: IntoIterator<Item = &'a S>,
    S: FileOps + 'a,
{
    for file in files_to_delete {
        let mut path = PathBuf::from(&location);
        path.push(file.path());

        file.remove(&path);
    }
}

/// Sorts (unstable) file paths in descending order by number of components, in parallel
///
/// # Arguments
/// `files_to_sort`: files to sort
///
/// # Returns
/// A vector of file paths in descending order by number of components
///
/// # Examples
/// ```
/// ["a", "a/b", "a/b/c"] becomes ["a/b/c", "a/b", "a"]
/// ["/usr", "/", "/usr/bin", "/etc"] becomes ["/usr/bin", "/usr", "/etc", "/"]
/// ```
pub fn sort_files<'a, T, S>(files_to_sort: T) -> Vec<&'a S>
where
    T: ParallelIterator<Item = &'a S>,
    S: FileOps + Sync + 'a,
{
    let mut files_to_sort = Vec::from_par_iter(files_to_sort);
    files_to_sort.par_sort_unstable_by(|a, b| {
        b.path()
            .components()
            .count()
            .cmp(&a.path().components().count())
    });
    files_to_sort
}

/// Generates a hash of the given file, using the Seahash non-cryptographic hash function
///
/// # Arguments
/// * `file_to_hash`: file object to hash
/// * `location`: base directory of the file to hash, such that
/// `location + file_to_hash.path()` is the absolute path of the file
///
/// # Returns
/// * Some: The hash of the given file
/// * Err: If the given file cannot be hashed
pub fn hash_file<S>(file_to_hash: &S, location: &str) -> Option<u64>
where
    S: FileOps,
{
    let mut file = PathBuf::from(&location);
    file.push(&file_to_hash.path());

    let contents = fs::read(file);
    if contents.is_err() {
        return None;
    }

    Some(hash(contents.unwrap().as_slice()))
}

/// Generates a hash of the given file, using the BLAKE2b cryptographic hash function
///
/// # Arguments
/// * `file_to_hash`: file object to hash
/// * `location`: base directory of the file to hash, such that
/// `location + file_to_hash.path()` is the absolute path of the file
///
/// # Returns
/// * Some: The hash of the given file
/// * Err: If the given file cannot be hashed
pub fn hash_file_secure<S>(file_to_hash: &S, location: &str) -> Option<Vec<u8>>
where
    S: FileOps,
{
    let mut file = PathBuf::from(&location);
    file.push(&file_to_hash.path());

    let file = fs::File::open(&file);
    if file.is_err() {
        eprintln!(
            "Error -- Opening File: {:?}: {}",
            file_to_hash.path(),
            file.err().unwrap()
        );
        return None;
    }

    let mut file = file.unwrap();
    let mut hasher = Blake2b::new();

    let hash = io::copy(&mut file, &mut hasher);

    if hash.is_err() {
        eprintln!(
            "Error -- Hashing: {:?}: {}",
            file_to_hash.path(),
            hash.err().unwrap()
        );
        return None;
    }

    Some(hasher.result().to_vec())
}

/// Recursively traverses a directory and all its subdirectories and returns
/// a FileSets that contains all files and all directories
///
/// # Arguments
/// * `src`: directory to traverse
///
/// # Returns
/// * Ok: A `FileSets` containing a set of files a set of directories
/// * Error: If `src` is an invalid directory
pub fn get_all_files(src: &str) -> Result<FileSets, io::Error> {
    get_all_files_helper(&PathBuf::from(&src), &src)
}

/// Recursive helper for `get_all_files`
///
/// # Arguments
/// * `src`: directory to traverse
/// * `base`: directory to traverse, used for recursive calls
///
/// # Returns
/// * Ok: A `FileSets` containing a set of files a set of directories
/// * Error: If `src` is an invalid directory
fn get_all_files_helper(src: &PathBuf, base: &str) -> Result<FileSets, io::Error> {
    let dir = src.read_dir()?;

    let mut files = HashSet::new();
    let mut dirs = HashSet::new();
    let mut symlinks = HashSet::new();

    for file in dir {
        if file.is_err() {
            eprintln!("{}", file.err().unwrap());
            continue;
        }

        let file = file.unwrap();
        let metadata = file.metadata();

        if metadata.is_err() {
            eprintln!(
                "Error -- Reading metadata of {:?} {}",
                file.path(),
                metadata.err().unwrap()
            );
            continue;
        }

        let metadata = metadata.unwrap();

        let path = file.path();
        // This is safe to unwrap, since `get_all_files` always calls this helper
        // with `base` equal to `src`
        let relative_path = path.strip_prefix(base).unwrap();

        if metadata.is_dir() {
            dirs.insert(Dir {
                path: relative_path.to_path_buf(),
            });

            // Recursively call `get_all_files_helper` on the subdirectory
            let file_sets = get_all_files_helper(&file.path(), base);

            if file_sets.is_err() {
                eprintln!("Error - Retrieving files: {}", file_sets.err().unwrap());
                continue;
            }
            let file_sets = file_sets.unwrap();

            // Add subdirectory subdirectories and files to sets
            files.extend(file_sets.files);
            dirs.extend(file_sets.dirs);
            symlinks.extend(file_sets.symlinks);
        } else if metadata.is_file() {
            files.insert(File {
                path: relative_path.to_path_buf(),
                size: metadata.len(),
            });
        } else {
            // If not a file nor dir, must be a symlink
            let target = fs::read_link(&path);
            if target.is_err() {
                eprintln!("Error - Reading symlink: {}", target.err().unwrap());
                continue;
            }
            let target = target.unwrap();
            symlinks.insert(Symlink {
                path: relative_path.to_path_buf(),
                target,
            });
        }
    }

    Ok(FileSets::with(files, dirs, symlinks))
}

#[cfg(test)]
mod test_get_all_files {
    use super::*;
    use std::process::Command;

    #[test]
    fn invalid_dir() {
        assert_eq!(get_all_files("/?").is_err(), true);
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn dir_insufficient_permissions() {
        assert_eq!(get_all_files("/root").is_err(), true);
    }

    #[test]
    fn empty_dir() {
        const TEST_DIR: &str = "test_get_all_files_empty_dir";

        fs::create_dir(TEST_DIR).unwrap();

        let file_sets = get_all_files(TEST_DIR).unwrap();

        assert_eq!(file_sets.files(), &HashSet::new());
        assert_eq!(file_sets.dirs(), &HashSet::new());

        fs::remove_dir(TEST_DIR).unwrap();
    }

    #[test]
    fn single_dir() {
        const TEST_DIR: &str = "test_get_all_files_single_dir";
        const TEST_SUB_DIR: &str = "test";

        fs::create_dir_all([TEST_DIR, TEST_SUB_DIR].join("/")).unwrap();

        let file_sets = get_all_files(&TEST_DIR).unwrap();
        let mut dir_set = HashSet::new();
        dir_set.insert(Dir {
            path: PathBuf::from(&TEST_SUB_DIR),
        });

        assert_eq!(file_sets.files(), &HashSet::new());
        assert_eq!(file_sets.dirs(), &dir_set);

        fs::remove_dir_all(&TEST_DIR).unwrap();
    }

    #[test]
    fn single_file() {
        const TEST_DIR: &str = "test_get_all_files_single_file";
        const TEST_FILE: &str = "file.txt";

        fs::create_dir_all(TEST_DIR).unwrap();

        fs::File::create([TEST_DIR, TEST_FILE].join("/")).unwrap();
        fs::write([TEST_DIR, TEST_FILE].join("/"), b"1234").unwrap();

        let file_sets = get_all_files(TEST_DIR).unwrap();
        let mut file_set = HashSet::new();
        file_set.insert(File {
            path: PathBuf::from(TEST_FILE),
            size: 4,
        });

        assert_eq!(file_sets.files(), &file_set);
        assert_eq!(file_sets.dirs(), &HashSet::new());

        fs::remove_dir_all(TEST_DIR).unwrap();
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn single_symlink() {
        use std::os::unix::fs::symlink;
        const TEST_DIR: &str = "test_get_all_files_single_symlink";
        const TEST_LINK: &str = "test_get_all_files_single_symlink/file";
        const TEST_FILE: &str = "test_get_all_files_single_symlink/test.txt";

        fs::create_dir_all(TEST_DIR).unwrap();
        symlink(TEST_FILE, TEST_LINK).unwrap();

        let mut symlink_set = HashSet::new();
        symlink_set.insert(Symlink {
            path: PathBuf::from("file"),
            target: PathBuf::from(TEST_FILE),
        });

        let file_sets = get_all_files(TEST_DIR).unwrap();

        assert_eq!(
            file_sets,
            FileSets {
                files: HashSet::new(),
                dirs: HashSet::new(),
                symlinks: symlink_set,
            }
        );

        fs::remove_dir_all(TEST_DIR).unwrap();
    }

    #[test]
    fn multi_level() {
        const TEST_DIR: &str = "test_get_all_files_multi_level";
        const SUB_DIRS: [&str; 2] = ["dir1", "dir1/dir2"];
        const TEST_FILES: [&str; 3] = ["file.txt", "dir1/file.txt", "dir1/dir2/file2.txt"];
        const TEST_DATA: [&[u8]; 3] = [b"1", b"", b"1234567890"];

        fs::create_dir_all([TEST_DIR, SUB_DIRS[1]].join("/")).unwrap();

        for i in 0..TEST_FILES.len() {
            let path = [TEST_DIR, TEST_FILES[i]].join("/");
            fs::File::create(&path).unwrap();
            fs::write(&path, TEST_DATA[i]).unwrap();
        }

        let file_sets = get_all_files(TEST_DIR).unwrap();
        let mut file_set = HashSet::new();
        let mut dir_set = HashSet::new();

        for i in 0..TEST_FILES.len() {
            file_set.insert(File {
                path: PathBuf::from(TEST_FILES[i]),
                size: TEST_DATA[i].len() as u64,
            });
        }

        for i in 0..SUB_DIRS.len() {
            dir_set.insert(Dir {
                path: PathBuf::from(SUB_DIRS[i]),
            });
        }

        assert_eq!(file_sets.files(), &file_set);
        assert_eq!(file_sets.dirs(), &dir_set);

        fs::remove_dir_all(TEST_DIR).unwrap();
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn multi_level_insufficient_permissions() {
        const TEST_DIR: &str = "test_get_all_files_multi_level_insufficient_permissions";
        const SUB_DIR: &str = "dir";
        const TEST_FILE: &str = "file.txt";

        let file_path = [TEST_DIR, TEST_FILE].join("/");
        let dir_path = [TEST_DIR, SUB_DIR].join("/");

        fs::create_dir_all(&dir_path).unwrap();
        fs::File::create(&file_path).unwrap();

        Command::new("chmod")
            .args(&["000", &file_path])
            .output()
            .unwrap();
        Command::new("chmod")
            .args(&["000", &dir_path])
            .output()
            .unwrap();

        let file_sets = get_all_files(TEST_DIR).unwrap();

        let mut file_set = HashSet::new();
        file_set.insert(File {
            path: PathBuf::from(&TEST_FILE),
            size: 0,
        });
        let mut dir_set = HashSet::new();
        dir_set.insert(Dir {
            path: PathBuf::from(&SUB_DIR),
        });

        assert_eq!(file_sets.files(), &file_set);
        assert_eq!(file_sets.dirs(), &dir_set);

        Command::new("chmod")
            .arg("777")
            .args(&["777", &dir_path])
            .output()
            .unwrap();
        fs::remove_dir_all(TEST_DIR).unwrap();
    }
}

#[cfg(test)]
mod test_sort_files {
    use super::*;

    #[test]
    fn no_dir() {
        let no_dir: HashSet<Dir> = HashSet::new();
        assert_eq!(sort_files(no_dir.par_iter()), Vec::<&Dir>::new());
    }

    #[test]
    fn single_dir() {
        let mut single_dir: HashSet<Dir> = HashSet::new();
        let dir = Dir {
            path: PathBuf::from("/"),
        };
        single_dir.insert(dir.clone());
        let expected: Vec<&Dir> = vec![&dir];

        assert_eq!(sort_files(single_dir.par_iter()), expected);
    }

    #[test]
    fn multi_dir_unique() {
        let mut multi_dir: HashSet<Dir> = HashSet::new();
        let dir1 = Dir {
            path: PathBuf::from("/"),
        };
        let dir2 = Dir {
            path: PathBuf::from("/a"),
        };
        let dir3 = Dir {
            path: PathBuf::from("/a/b"),
        };
        multi_dir.insert(dir1.clone());
        multi_dir.insert(dir2.clone());
        multi_dir.insert(dir3.clone());
        let expected: Vec<&Dir> = vec![&dir3, &dir2, &dir1];

        assert_eq!(sort_files(multi_dir.par_iter()), expected);
    }

    #[test]
    fn multi_dir() {
        let mut multi_dir: HashSet<Dir> = HashSet::new();
        let dir1 = Dir {
            path: PathBuf::from("/"),
        };
        let dir2 = Dir {
            path: PathBuf::from("/a/c"),
        };
        let dir3 = Dir {
            path: PathBuf::from("/a/b"),
        };
        multi_dir.insert(dir1.clone());
        multi_dir.insert(dir2.clone());
        multi_dir.insert(dir3.clone());
        let expected: Vec<&Dir> = vec![&dir2, &dir3, &dir1];

        assert_eq!(
            sort_files(multi_dir.par_iter()).get(2).unwrap(),
            &expected[2]
        );
    }
}

#[cfg(test)]
mod test_hash_file {
    use super::*;

    #[test]
    fn invalid_file() {
        assert_eq!(
            hash_file(
                &File {
                    path: PathBuf::from("test"),
                    size: 0,
                },
                "."
            ),
            None
        );
    }

    #[test]
    fn empty_file() {
        const TEST_FILE1: &str = "test_hash_file_empty_file1.txt";
        const TEST_FILE2: &str = "test_hash_file_empty_file2.txt";

        fs::File::create(TEST_FILE1).unwrap();
        fs::File::create(TEST_FILE2).unwrap();

        assert_eq!(
            hash_file(
                &File {
                    path: PathBuf::from(TEST_FILE1),
                    size: 0,
                },
                "."
            ),
            hash_file(
                &File {
                    path: PathBuf::from(TEST_FILE2),
                    size: 0,
                },
                "."
            )
        );
        assert_eq!(
            hash_file_secure(
                &File {
                    path: PathBuf::from(TEST_FILE1),
                    size: 0,
                },
                "."
            ),
            hash_file_secure(
                &File {
                    path: PathBuf::from(TEST_FILE2),
                    size: 0,
                },
                "."
            )
        );

        fs::remove_file(TEST_FILE1).unwrap();
        fs::remove_file(TEST_FILE2).unwrap();
    }

    #[test]
    fn equal_files() {
        const TEST_DIR: &str = "test_hash_file_equal_files";
        const TEST_FILE1: &str = "file1.txt";
        const TEST_FILE2: &str = "file2.txt";

        let path1 = [TEST_DIR, TEST_FILE1].join("/");
        let path2 = [TEST_DIR, TEST_FILE2].join("/");

        fs::create_dir_all(TEST_DIR).unwrap();
        fs::File::create(&path1).unwrap();
        fs::File::create(&path2).unwrap();
        fs::write(path1, b"1234567890").unwrap();
        fs::write(path2, b"1234567890").unwrap();

        assert_eq!(
            hash_file(
                &File {
                    path: PathBuf::from(TEST_FILE1),
                    size: 10,
                },
                "."
            ),
            hash_file(
                &File {
                    path: PathBuf::from(TEST_FILE2),
                    size: 10,
                },
                "."
            )
        );
        assert_eq!(
            hash_file_secure(
                &File {
                    path: PathBuf::from(TEST_FILE1),
                    size: 10,
                },
                "."
            ),
            hash_file_secure(
                &File {
                    path: PathBuf::from(TEST_FILE2),
                    size: 10,
                },
                "."
            )
        );

        fs::remove_dir_all(TEST_DIR).unwrap();
    }

    #[test]
    fn different_files() {
        assert_ne!(
            hash_file(
                &File {
                    path: PathBuf::from("lumins/file_ops.rs"),
                    size: 0,
                },
                "src"
            ),
            hash_file(
                &File {
                    path: PathBuf::from("main.rs"),
                    size: 0,
                },
                "src"
            )
        );
        assert_ne!(
            hash_file_secure(
                &File {
                    path: PathBuf::from("lumins/file_ops.rs"),
                    size: 0,
                },
                "src"
            ),
            hash_file_secure(
                &File {
                    path: PathBuf::from("main.rs"),
                    size: 0,
                },
                "src"
            )
        );
    }
}

#[cfg(test)]
mod test_delete_files {
    use super::*;

    #[test]
    fn delete_no_files() {
        const TEST_DIR: &str = "test_delete_files_delete_no_files";
        const TEST_FILES: [&str; 2] = ["file1.txt", "file2.txt"];

        fs::create_dir_all(TEST_DIR).unwrap();

        let files_to_delete: HashSet<File> = HashSet::new();
        let files_to_delete_sequential: Vec<&File> = Vec::new();
        let mut file_set = HashSet::new();

        for i in 0..TEST_FILES.len() {
            fs::File::create([TEST_DIR, TEST_FILES[i]].join("/")).unwrap();
            let file = File {
                path: PathBuf::from(TEST_FILES[i]),
                size: 0,
            };
            file_set.insert(file);
        }

        delete_files(files_to_delete.par_iter(), TEST_DIR);
        delete_files_sequential(files_to_delete_sequential.into_iter(), TEST_DIR);

        assert_eq!(
            get_all_files(TEST_DIR).unwrap(),
            FileSets {
                files: file_set,
                dirs: HashSet::new(),
                symlinks: HashSet::new(),
            }
        );

        fs::remove_dir_all(TEST_DIR).unwrap();
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn delete_invalid_file_and_link() {
        use std::os::unix::fs::symlink;

        const TEST_DIR: &str = "test_delete_files_delete_invalid_file_and_link";
        const TEST_DIR_SEQ: &str = "test_delete_files_delete_invalid_file_and_link_seq";
        const TEST_FILES: [&str; 2] = ["file1.txt", "file2.txt"];

        fs::create_dir_all(TEST_DIR).unwrap();
        fs::create_dir_all(TEST_DIR_SEQ).unwrap();

        let mut files_to_delete: HashSet<File> = HashSet::new();
        let mut files_to_delete_sequential: Vec<&File> = Vec::new();
        let mut file_set = HashSet::new();

        fs::File::create([TEST_DIR, TEST_FILES[0]].join("/")).unwrap();
        fs::File::create([TEST_DIR_SEQ, TEST_FILES[0]].join("/")).unwrap();
        let file = File {
            path: PathBuf::from([TEST_FILES[0], "a"].join("/")),
            size: 0,
        };
        let expected_file = File {
            path: PathBuf::from(TEST_FILES[0]),
            size: 0,
        };
        file_set.insert(expected_file);
        files_to_delete.insert(file.clone());
        files_to_delete_sequential.push(&file);

        let mut links_to_delete: HashSet<Symlink> = HashSet::new();
        let mut links_to_delete_sequential: Vec<&Symlink> = Vec::new();
        let mut link_set = HashSet::new();

        symlink(TEST_FILES[1], [TEST_DIR, "file"].join("/")).unwrap();
        symlink(TEST_FILES[1], [TEST_DIR_SEQ, "file"].join("/")).unwrap();
        let link = Symlink {
            path: PathBuf::from("filea"),
            target: PathBuf::from(TEST_FILES[1]),
        };
        let expected_link = Symlink {
            path: PathBuf::from("file"),
            target: PathBuf::from(TEST_FILES[1]),
        };
        link_set.insert(expected_link);
        links_to_delete.insert(link.clone());
        links_to_delete_sequential.push(&link);

        delete_files(files_to_delete.par_iter(), TEST_DIR);
        delete_files_sequential(files_to_delete_sequential.into_iter(), TEST_DIR_SEQ);
        delete_files(links_to_delete.par_iter(), TEST_DIR);
        delete_files_sequential(links_to_delete_sequential.into_iter(), TEST_DIR_SEQ);

        assert_eq!(
            get_all_files(TEST_DIR).unwrap(),
            FileSets {
                files: file_set.clone(),
                dirs: HashSet::new(),
                symlinks: link_set.clone(),
            }
        );
        assert_eq!(
            get_all_files(TEST_DIR_SEQ).unwrap(),
            FileSets {
                files: file_set,
                dirs: HashSet::new(),
                symlinks: link_set,
            }
        );

        fs::remove_dir_all(TEST_DIR).unwrap();
        fs::remove_dir_all(TEST_DIR_SEQ).unwrap();
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn delete_file_and_link() {
        use std::os::unix::fs::symlink;

        const TEST_DIR: &str = "test_delete_files_delete_file_and_link";
        const TEST_DIR_SEQ: &str = "test_delete_files_delete_file_and_link_seq";
        const TEST_FILES: [&str; 2] = ["file1.txt", "file2.txt"];

        fs::create_dir_all(TEST_DIR).unwrap();
        fs::create_dir_all(TEST_DIR_SEQ).unwrap();

        let mut files_to_delete: HashSet<File> = HashSet::new();
        let mut files_to_delete_sequential: Vec<&File> = Vec::new();
        let mut file_set = HashSet::new();

        fs::File::create([TEST_DIR, TEST_FILES[0]].join("/")).unwrap();
        fs::File::create([TEST_DIR_SEQ, TEST_FILES[0]].join("/")).unwrap();
        let file = File {
            path: PathBuf::from(TEST_FILES[0]),
            size: 0,
        };
        file_set.insert(file.clone());
        files_to_delete.insert(file.clone());
        files_to_delete_sequential.push(&file);

        let mut links_to_delete: HashSet<Symlink> = HashSet::new();
        let mut links_to_delete_sequential: Vec<&Symlink> = Vec::new();
        let mut link_set = HashSet::new();

        symlink(TEST_FILES[1], [TEST_DIR, "file"].join("/")).unwrap();
        symlink(TEST_FILES[1], [TEST_DIR_SEQ, "file"].join("/")).unwrap();
        let link = Symlink {
            path: PathBuf::from("file"),
            target: PathBuf::from(TEST_FILES[1]),
        };
        link_set.insert(link.clone());
        links_to_delete.insert(link.clone());
        links_to_delete_sequential.push(&link);

        delete_files(files_to_delete.par_iter(), TEST_DIR);
        delete_files_sequential(files_to_delete_sequential.into_iter(), TEST_DIR_SEQ);
        delete_files(links_to_delete.par_iter(), TEST_DIR);
        delete_files_sequential(links_to_delete_sequential.into_iter(), TEST_DIR_SEQ);

        assert_eq!(
            get_all_files(TEST_DIR).unwrap(),
            FileSets {
                files: HashSet::new(),
                dirs: HashSet::new(),
                symlinks: HashSet::new(),
            }
        );
        assert_eq!(
            get_all_files(TEST_DIR_SEQ).unwrap(),
            FileSets {
                files: HashSet::new(),
                dirs: HashSet::new(),
                symlinks: HashSet::new(),
            }
        );

        fs::remove_dir_all(TEST_DIR).unwrap();
        fs::remove_dir_all(TEST_DIR_SEQ).unwrap();
    }

    #[test]
    fn delete_partial_dirs() {
        const TEST_DIR: &str = "test_delete_files_delete_partial_dirs";
        const TEST_DIR_SEQ: &str = "test_delete_files_delete_partial_dirs_seq";
        const TEST_SUB_DIRS: [&str; 3] = ["dir0", "dir1", "dir2"];

        fs::create_dir_all([TEST_DIR, TEST_SUB_DIRS[0], TEST_SUB_DIRS[1]].join("/")).unwrap();
        fs::create_dir_all([TEST_DIR_SEQ, TEST_SUB_DIRS[0], TEST_SUB_DIRS[1]].join("/")).unwrap();
        fs::create_dir_all([TEST_DIR, TEST_SUB_DIRS[2]].join("/")).unwrap();
        fs::create_dir_all([TEST_DIR_SEQ, TEST_SUB_DIRS[2]].join("/")).unwrap();

        let mut dirs_to_delete: HashSet<Dir> = HashSet::new();
        let mut dirs_to_delete_sequential: Vec<&Dir> = Vec::new();
        let mut file_set: HashSet<Dir> = HashSet::new();

        let dir0 = Dir {
            path: PathBuf::from(TEST_SUB_DIRS[0]),
        };
        let dir2 = Dir {
            path: PathBuf::from(TEST_SUB_DIRS[2]),
        };

        dirs_to_delete.insert(dir0.clone());
        dirs_to_delete.insert(dir2.clone());
        dirs_to_delete_sequential.push(&dir0);
        dirs_to_delete_sequential.push(&dir2);

        delete_files(dirs_to_delete.par_iter(), TEST_DIR);
        delete_files_sequential(dirs_to_delete_sequential.into_iter(), TEST_DIR_SEQ);

        file_set.insert(Dir {
            path: PathBuf::from(TEST_SUB_DIRS[0]),
        });
        file_set.insert(Dir {
            path: PathBuf::from([TEST_SUB_DIRS[0], TEST_SUB_DIRS[1]].join("/")),
        });

        assert_eq!(
            get_all_files(TEST_DIR).unwrap(),
            FileSets {
                files: HashSet::new(),
                dirs: file_set.clone(),
                symlinks: HashSet::new(),
            }
        );
        assert_eq!(
            get_all_files(TEST_DIR_SEQ).unwrap(),
            FileSets {
                files: HashSet::new(),
                dirs: file_set,
                symlinks: HashSet::new(),
            }
        );

        fs::remove_dir_all(TEST_DIR).unwrap();
        fs::remove_dir_all(TEST_DIR_SEQ).unwrap();
    }
}

#[cfg(test)]
mod test_copy_files {
    use super::*;
    use std::process::Command;

    #[test]
    fn no_files() {
        const TEST_DIR: &str = "test_copy_files_no_files";
        const TEST_DIR_OUT: &str = "test_copy_files_no_files_out";

        fs::create_dir_all(TEST_DIR).unwrap();
        fs::create_dir_all(TEST_DIR_OUT).unwrap();

        copy_files(HashSet::<File>::new().par_iter(), TEST_DIR, TEST_DIR_OUT);

        assert_eq!(
            get_all_files(TEST_DIR_OUT).unwrap(),
            FileSets {
                files: HashSet::new(),
                dirs: HashSet::new(),
                symlinks: HashSet::new(),
            }
        );

        fs::remove_dir_all(TEST_DIR).unwrap();
        fs::remove_dir_all(TEST_DIR_OUT).unwrap();
    }

    #[test]
    fn regular_files_dirs() {
        const TEST_DIR: &str = "src";
        const TEST_DIR_OUT: &str = "test_copy_files_regular_files_dirs_out";

        fs::create_dir_all(TEST_DIR_OUT).unwrap();

        copy_files(
            get_all_files(TEST_DIR).unwrap().dirs().par_iter(),
            TEST_DIR,
            TEST_DIR_OUT,
        );
        copy_files(
            get_all_files(TEST_DIR).unwrap().files().par_iter(),
            TEST_DIR,
            TEST_DIR_OUT,
        );

        assert_eq!(
            get_all_files(TEST_DIR_OUT).unwrap(),
            get_all_files(TEST_DIR).unwrap()
        );

        fs::remove_dir_all(TEST_DIR_OUT).unwrap();
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn insufficient_output_permissions() {
        const TEST_DIR: &str = "src";
        const TEST_DIR_OUT: &str = "test_copy_files_insufficient_output_permissions_out";
        const SUB_DIR: &str = "lumins";

        fs::create_dir_all([TEST_DIR_OUT, SUB_DIR].join("/")).unwrap();
        fs::File::create([TEST_DIR_OUT, "main.rs"].join("/")).unwrap();
        fs::File::create([TEST_DIR_OUT, "cli.yml"].join("/")).unwrap();
        Command::new("chmod")
            .arg("000")
            .arg([TEST_DIR_OUT, SUB_DIR].join("/"))
            .output()
            .unwrap();
        Command::new("chmod")
            .arg("000")
            .arg([TEST_DIR_OUT, "main.rs"].join("/"))
            .output()
            .unwrap();
        Command::new("chmod")
            .arg("000")
            .arg([TEST_DIR_OUT, "cli.yml"].join("/"))
            .output()
            .unwrap();

        copy_files(
            get_all_files(TEST_DIR).unwrap().dirs().par_iter(),
            TEST_DIR,
            TEST_DIR_OUT,
        );
        copy_files(
            get_all_files(TEST_DIR).unwrap().files().par_iter(),
            TEST_DIR,
            TEST_DIR_OUT,
        );

        let mut files = HashSet::new();
        files.insert(File {
            path: PathBuf::from("main.rs"),
            size: 0,
        });
        files.insert(File {
            path: PathBuf::from("cli.yml"),
            size: 0,
        });
        let mut dirs = HashSet::new();
        dirs.insert(Dir {
            path: PathBuf::from("lumins"),
        });

        assert_eq!(
            get_all_files(TEST_DIR_OUT).unwrap(),
            FileSets {
                files,
                dirs,
                symlinks: HashSet::new(),
            }
        );

        Command::new("rm")
            .arg("-rf")
            .arg(TEST_DIR_OUT)
            .output()
            .unwrap();
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn insufficient_input_permissions() {
        const TEST_DIR: &str = "test_copy_files_insufficient_input_permissions";
        const TEST_DIR_OUT: &str = "test_copy_files_insufficient_input_permissions_out";

        fs::create_dir_all(TEST_DIR).unwrap();
        fs::create_dir_all(TEST_DIR_OUT).unwrap();
        Command::new("cp")
            .args(&["-r", "src/lumins", TEST_DIR])
            .output()
            .unwrap();
        Command::new("cp")
            .args(&["src/main.rs", TEST_DIR])
            .output()
            .unwrap();
        Command::new("chmod")
            .arg("000")
            .arg([TEST_DIR, "lumins"].join("/"))
            .output()
            .unwrap();
        Command::new("chmod")
            .arg("000")
            .arg([TEST_DIR, "main.rs"].join("/"))
            .output()
            .unwrap();

        copy_files(
            get_all_files(TEST_DIR).unwrap().dirs().par_iter(),
            TEST_DIR,
            TEST_DIR_OUT,
        );
        copy_files(
            get_all_files(TEST_DIR).unwrap().files().par_iter(),
            TEST_DIR,
            TEST_DIR_OUT,
        );

        let files = HashSet::new();
        let mut dirs = HashSet::new();
        dirs.insert(Dir {
            path: PathBuf::from("lumins"),
        });

        assert_eq!(
            get_all_files(TEST_DIR_OUT).unwrap(),
            FileSets {
                files,
                dirs,
                symlinks: HashSet::new(),
            }
        );

        Command::new("chmod")
            .arg("777")
            .arg([TEST_DIR, "lumins"].join("/"))
            .output()
            .unwrap();
        Command::new("rm")
            .args(&["-rf", TEST_DIR])
            .output()
            .unwrap();
        Command::new("rm")
            .args(&["-rf", TEST_DIR_OUT])
            .output()
            .unwrap();
    }

    #[test]
    #[cfg(target_family = "unix")]
    fn copy_symlink() {
        use std::os::unix::fs::symlink;
        const TEST_DIR: &str = "test_copy_files_copy_symlink";
        const TEST_DIR_OUT: &str = "test_copy_files_copy_symlink_out";

        fs::create_dir_all(TEST_DIR).unwrap();
        fs::create_dir_all(TEST_DIR_OUT).unwrap();
        symlink("src/main.rs", [TEST_DIR, "file"].join("/")).unwrap();

        copy_files(
            get_all_files(TEST_DIR).unwrap().dirs().par_iter(),
            TEST_DIR,
            TEST_DIR_OUT,
        );
        copy_files(
            get_all_files(TEST_DIR).unwrap().files().par_iter(),
            TEST_DIR,
            TEST_DIR_OUT,
        );
        copy_files(
            get_all_files(TEST_DIR).unwrap().symlinks().par_iter(),
            TEST_DIR,
            TEST_DIR_OUT,
        );

        let mut links_set = HashSet::new();
        links_set.insert(Symlink {
            path: PathBuf::from("file"),
            target: PathBuf::from("src/main.rs"),
        });

        assert_eq!(
            get_all_files(TEST_DIR_OUT).unwrap(),
            FileSets {
                files: HashSet::new(),
                dirs: HashSet::new(),
                symlinks: links_set,
            }
        );

        fs::remove_dir_all(TEST_DIR).unwrap();
        fs::remove_dir_all(TEST_DIR_OUT).unwrap();
    }
}

#[cfg(test)]
mod test_compare_and_copy_files {
    use super::*;

    #[test]
    fn single_same() {
        const TEST_DIR: &str = "src";
        const TEST_DIR_OUT: &str = "test_compare_and_copy_files_single_same_out";
        fs::create_dir_all(TEST_DIR_OUT).unwrap();
        fs::copy(
            [TEST_DIR, "main.rs"].join("/"),
            [TEST_DIR_OUT, "main.rs"].join("/"),
        )
        .unwrap();

        let mut files_to_compare = HashSet::new();
        files_to_compare.insert(File {
            path: PathBuf::from("main.rs"),
            size: fs::metadata([TEST_DIR, "main.rs"].join("/")).unwrap().len(),
        });
        compare_and_copy_files(files_to_compare.par_iter(), TEST_DIR, TEST_DIR_OUT, 0);
        compare_and_copy_files(
            files_to_compare.par_iter(),
            TEST_DIR,
            TEST_DIR_OUT,
            parse::Flag::Secure as u32,
        );
        let actual = fs::read([TEST_DIR_OUT, "main.rs"].join("/")).unwrap();
        let expected = fs::read([TEST_DIR, "main.rs"].join("/")).unwrap();
        assert_eq!(expected, actual);

        fs::remove_dir_all(TEST_DIR_OUT).unwrap();
    }

    #[test]
    fn single_different() {
        const TEST_DIR: &str = "src";
        const TEST_DIR_OUT: &str = "test_compare_and_copy_files_single_different_out";
        fs::create_dir_all(TEST_DIR_OUT).unwrap();
        fs::File::create([TEST_DIR_OUT, "main.rs"].join("/")).unwrap();

        let mut files_to_compare = HashSet::new();
        files_to_compare.insert(File {
            path: PathBuf::from("main.rs"),
            size: fs::metadata([TEST_DIR, "main.rs"].join("/")).unwrap().len(),
        });
        compare_and_copy_files(files_to_compare.par_iter(), TEST_DIR, TEST_DIR_OUT, 0);
        let actual = fs::read([TEST_DIR_OUT, "main.rs"].join("/")).unwrap();
        let expected = fs::read([TEST_DIR, "main.rs"].join("/")).unwrap();
        assert_eq!(expected, actual);

        fs::remove_dir_all(TEST_DIR_OUT).unwrap();
    }
}

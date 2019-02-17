use std::marker::Sync;
use std::path::PathBuf;
use std::{fs, io};

use blake2::{Blake2b, Digest};
use rayon::prelude::*;
use rayon_hash::HashSet;

pub trait FileOps {
    fn path(&self) -> &PathBuf;
    fn remove(&self, path: &PathBuf);
    fn copy(&self, src: &PathBuf, dest: &PathBuf);
}

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
        }
    }
    fn copy(&self, src: &PathBuf, dest: &PathBuf) {
        let copy = fs::copy(&src, &dest);
        if copy.is_err() {
            eprintln!("Error -- Copying {:?} {}", src, copy.err().unwrap());
        }
    }
}

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
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct FileSets {
    files: HashSet<File>,
    dirs: HashSet<Dir>,
}

impl FileSets {
    pub fn new() -> Self {
        FileSets {
            files: HashSet::new(),
            dirs: HashSet::new(),
        }
    }
    pub fn with(files: HashSet<File>, dirs: HashSet<Dir>) -> Self {
        FileSets { files, dirs }
    }
    pub fn files(&self) -> &HashSet<File> {
        &self.files
    }
    pub fn dirs(&self) -> &HashSet<Dir> {
        &self.dirs
    }
}

pub fn compare_files<'a, T, S>(files_to_compare: T, src: &str, dest: &str)
where
    T: ParallelIterator<Item = &'a S>,
    S: FileOps + Sync + 'a,
{
    files_to_compare.for_each(|file| {
        if !compare_file(file, &src, &dest) {
            copy_file(file, &src, &dest);
        }
    });
}

fn compare_file<S>(file_to_compare: &S, src: &str, dest: &str) -> bool
where
    S: FileOps,
{
    let src_file_hash = hash_file(file_to_compare, &src);
    let dest_file_hash = hash_file(file_to_compare, &dest);

    src_file_hash.is_some() && (src_file_hash == dest_file_hash)
}

pub fn copy_files<'a, T, S>(files_to_copy: T, src: &str, dest: &str)
where
    T: ParallelIterator<Item = &'a S>,
    S: FileOps + Sync + 'a,
{
    files_to_copy.for_each(|file| {
        copy_file(file, &src, &dest);
    });
}

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

/// Sorts (unstable) file paths in descending order by number of components
///
/// # Arguments
/// * `files_to_sort`: files to sort
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

/// Generates a hash of the given file, using the BLAKE2b hash function
///
/// # Arguments
/// * `file_to_hash`: file object to hash
/// * `location`: base directory of the file to hash, such that
/// `location + file_to_hash.path()` is the absolute path of the file
///
/// # Returns
/// * Some: The hash of the given file
/// * Err: If the given file cannot be hashed
pub fn hash_file<S>(file_to_hash: &S, location: &str) -> Option<Vec<u8>>
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

        // Only copy files and dirs -- no symlinks
        if !metadata.is_dir() && !metadata.is_file() {
            continue;
        }

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
        } else {
            // if file is a file
            files.insert(File {
                path: relative_path.to_path_buf(),
                size: metadata.len(),
            });
        }
    }

    Ok(FileSets::with(files, dirs))
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

        assert_eq!(sort_files(multi_dir.par_iter()).get(2).unwrap(), &expected[2]);
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
    }
}

#[cfg(test)]
mod test_get_all_files {
    use super::*;
    use std::os::unix::fs::symlink;
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
        const TEST_DIR: &str = "test_get_all_files_single_symlink";
        const TEST_LINK: &str = "test_get_all_files_single_symlink/file";
        const TEST_FILE: &str = "test_get_all_files_single_symlink/test.txt";

        fs::create_dir_all(TEST_DIR).unwrap();
        symlink(TEST_FILE, TEST_LINK).unwrap();

        let file_sets = get_all_files(TEST_DIR).unwrap();

        assert_eq!(file_sets.files(), &HashSet::new());
        assert_eq!(file_sets.dirs(), &HashSet::new());

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
            .arg("000")
            .arg(&file_path)
            .output()
            .unwrap();
        Command::new("chmod")
            .arg("000")
            .arg(&dir_path)
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
            .arg(&dir_path)
            .output()
            .unwrap();
        fs::remove_dir_all(TEST_DIR).unwrap();
    }
}

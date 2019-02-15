use std::marker::Sync;
use std::path::PathBuf;
use std::{fs, io};

use blake2::{Blake2b, Digest};
use rayon::prelude::*;
use rayon_hash::HashSet;

trait FileOps {
    fn path(&self) -> &PathBuf;
    fn remove(&self, path: &PathBuf) -> Result<(), io::Error>;
    fn copy(&self, src: &PathBuf, dest: &PathBuf);
}

#[derive(Hash, Eq, PartialEq)]
struct File {
    path: PathBuf,
    size: u64,
}

impl FileOps for File {
    fn path(&self) -> &PathBuf {
        &self.path
    }
    fn remove(&self, path: &PathBuf) -> Result<(), io::Error> {
        fs::remove_file(&path)
    }
    fn copy(&self, src: &PathBuf, dest: &PathBuf) {
        let copy = fs::copy(&src, &dest);
        if copy.is_err() {
            eprintln!("Error -- Copying {:?} {}", src, copy.err().unwrap());
        }
    }
}

#[derive(Hash, Eq, PartialEq)]
struct Dir {
    path: PathBuf,
}

impl FileOps for Dir {
    fn path(&self) -> &PathBuf {
        &self.path
    }
    fn remove(&self, path: &PathBuf) -> Result<(), io::Error> {
        fs::remove_dir_all(&path)
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

pub fn synchronize(src: &String, dest: &String) {
    let (src_files, src_dirs) = get_all_files(&PathBuf::from(&src), &src);
    let (dest_files, dest_dirs) = get_all_files(&PathBuf::from(&dest), &dest);

    let dirs_to_delete = dest_dirs.par_difference(&src_dirs);
    let dirs_to_copy = src_dirs.par_difference(&dest_dirs);

    copy_files(dirs_to_copy, &src, &dest);

    let files_to_delete = dest_files.par_difference(&src_files);
    let files_to_copy = src_files.par_difference(&dest_files);
    let files_to_compare = src_files.par_intersection(&dest_files);

    delete_files(files_to_delete, &dest);
    copy_files(files_to_copy, &src, &dest);
    compare_files(files_to_compare, &src, &dest);

    delete_files(dirs_to_delete, &dest);
}

fn compare_files<'a, T, S>(files_to_compare: T, src: &String, dest: &String)
where
    T: ParallelIterator<Item = &'a S>,
    S: FileOps + Sync + 'a,
{
    files_to_compare.for_each(|file| {
        let same = compare_file(file, &src, &dest);
        if !same {
            copy_file(file, &src, &dest);
        }
    });
}

fn compare_file<'a, S>(file_to_compare: &S, src: &String, dest: &String) -> bool
where
    S: FileOps,
{
    let src_file_hash = hash_file(file_to_compare, &src);
    let dest_file_hash = hash_file(file_to_compare, &dest);

    if src_file_hash.is_none() || dest_file_hash.is_none() {
        return false;
    }

    src_file_hash == dest_file_hash
}

fn copy_files<'a, T, S>(files_to_copy: T, src: &String, dest: &String)
where
    T: ParallelIterator<Item = &'a S>,
    S: FileOps + Sync + 'a,
{
    files_to_copy.for_each(|file| {
        copy_file(file, &src, &dest);
    });
}

fn copy_file<S>(file_to_copy: &S, src: &String, dest: &String)
where
    S: FileOps,
{
    let mut src_file = PathBuf::from(&src);
    src_file.push(&file_to_copy.path());
    let mut dest_file = PathBuf::from(&dest);
    dest_file.push(&file_to_copy.path());

    file_to_copy.copy(&src_file, &dest_file);
}

fn delete_files<'a, T, S>(files_to_delete: T, location: &String)
where
    T: ParallelIterator<Item = &'a S>,
    S: FileOps + Sync + 'a,
{
    files_to_delete.for_each(|file| {
        let mut path = PathBuf::from(&location);
        path.push(file.path());

        let delete_file: Result<(), io::Error> = file.remove(&path);
        if delete_file.is_err() {
            eprintln!(
                "Error -- Deleting {:?}: {}",
                file.path(),
                delete_file.err().unwrap()
            );
        }
    });
}

fn hash_file<S>(file_to_hash: &S, location: &String) -> Option<Vec<u8>>
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

    let mut hasher = Blake2b::new();
    let mut file = file.unwrap();

    let hashing = io::copy(&mut file, &mut hasher);

    if hashing.is_err() {
        eprintln!(
            "Error -- Hashing: {:?}: {}",
            file_to_hash.path(),
            hashing.err().unwrap()
        );
        return None;
    }

    Some(hasher.result().to_vec())
}

fn get_all_files(src: &PathBuf, base: &String) -> (HashSet<File>, HashSet<Dir>) {
    let dir = src.read_dir();
    let dir = match dir {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}", e);
            return (HashSet::new(), HashSet::new());
        }
    };

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

        if !metadata.is_dir() && !metadata.is_file() {
            continue;
        }

        let path = file.path();
        let relative_path = path.strip_prefix(base);
        if relative_path.is_err() {
            eprintln!("Error -- Stripping base: {}", relative_path.err().unwrap());
            continue;
        }

        let relative_path = relative_path.unwrap();

        if metadata.is_dir() {
            dirs.insert(Dir {
                path: relative_path.to_path_buf(),
            });
            let (sub_files, sub_dirs) = get_all_files(&file.path(), base);
            files.extend(sub_files);
            dirs.extend(sub_dirs);
        } else {
            files.insert(File {
                path: relative_path.to_path_buf(),
                size: metadata.len(),
            });
        }
    }

    (files, dirs)
}

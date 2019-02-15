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

#[derive(Hash, Eq, PartialEq)]
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

#[derive(Hash, Eq, PartialEq)]
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

fn hash_file<S>(file_to_hash: &S, location: &str) -> Option<Vec<u8>>
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

pub fn get_all_files(src: &PathBuf, base: &str) -> FileSets {
    let dir = src.read_dir();
    if dir.is_err() {
        eprintln!("{}", dir.err().unwrap());
        return FileSets::new();
    }

    let dir = dir.ok().unwrap();

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
            let file_sets = get_all_files(&file.path(), base);
            files.extend(file_sets.files);
            dirs.extend(file_sets.dirs);
        } else {
            files.insert(File {
                path: relative_path.to_path_buf(),
                size: metadata.len(),
            });
        }
    }

    FileSets::with(files, dirs)
}

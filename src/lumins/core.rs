use std::path::PathBuf;
use std::{fs, io};

use blake2::{Blake2b, Digest};
use rayon::prelude::*;

const CHUNK_SIZE: usize = 256;

struct HashedFile {
    path: PathBuf,
    hash: Option<Vec<u8>>,
}

pub fn synchronize(src: &String, dest: &String) {
    let mut files = get_all_files(&PathBuf::from(src));
    hash_files(&mut files, CHUNK_SIZE);

    copy_files(&files, src, dest, CHUNK_SIZE);
}

fn copy_files(files: &Vec<HashedFile>, src: &String, dest: &String, chunk_size: usize) {
    files.par_chunks(chunk_size).for_each(|chunk| {
        for file in chunk {
            let mut dest_file = PathBuf::from(dest);
            let rest = match file.path.strip_prefix(src) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Error -- Stripping Prefix: {}", e);
                    continue;
                }
            };

            dest_file.push(rest);
            if dest_file.parent().is_some() {
                let create_parent_dir = fs::create_dir_all(dest_file.parent().unwrap());
                if create_parent_dir.is_err() {
                    eprintln!(
                        "Error -- Creating Directory {:?}: {}",
                        dest_file.parent().unwrap(),
                        create_parent_dir.err().unwrap()
                    );
                }
            }

            let copy = fs::copy(&file.path, &dest_file);
            if copy.is_err() {
                eprintln!(
                    "Error -- Copying {:?} to {:?}: {}",
                    file.path,
                    dest_file,
                    copy.err().unwrap()
                );
            }
        }
    });
}

fn hash_files(hashed_files: &mut Vec<HashedFile>, chunk_size: usize) {
    hashed_files.par_chunks_mut(chunk_size).for_each(|chunk| {
        for hashed_file in chunk.iter_mut() {
            let file = fs::File::open(&hashed_file.path);
            if file.is_err() {
                eprintln!(
                    "Error -- Opening File: {:?}: {}",
                    hashed_file.path,
                    file.err().unwrap()
                );
                continue;
            }

            let mut hasher = Blake2b::new();
            let mut file = file.unwrap();

            let hashing = io::copy(&mut file, &mut hasher);

            if hashing.is_err() {
                eprintln!(
                    "Error -- Hashing: {:?}: {}",
                    hashed_file.path,
                    hashing.err().unwrap()
                );
                continue;
            }

            hashed_file.hash = Some(hasher.result().to_vec());
        }
    });
}

fn get_all_files(src: &PathBuf) -> Vec<HashedFile> {
    let dir = src.read_dir();
    let dir = match dir {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}", e);
            return Vec::new();
        }
    };

    let mut files = Vec::new();

    for file in dir {
        if file.is_err() {
            eprintln!("{}", file.err().unwrap());
            continue;
        }

        let file = file.unwrap();
        let metadata = file.metadata();

        if metadata.is_err() {
            eprintln!("{:?} {}", file.path(), metadata.err().unwrap());
            continue;
        }

        let metadata = metadata.unwrap();

        if metadata.is_dir() {
            files.extend(get_all_files(&file.path()));
        } else if metadata.is_file() {
            files.push(HashedFile {
                path: file.path(),
                hash: None,
            });
        }
    }

    files
}

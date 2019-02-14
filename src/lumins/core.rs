use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;

const NUM_THREADS: usize = 10;

pub fn synchronize(src: &String, dest: &String) {
    let files = get_all_files(&PathBuf::from(src));

    copy_files(&files, src, dest, NUM_THREADS);
}

fn copy_files(files: &Vec<PathBuf>, src: &String, dest: &String, num_threads: usize) {
    files.par_chunks(256).for_each(|slice| {
        for file in slice {
            let mut dest_file = PathBuf::from(dest);
            let rest = match file.strip_prefix(src) {
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

            let copy = fs::copy(&file, &dest_file);
            if copy.is_err() {
                eprintln!(
                    "Error -- Copying {:?} to {:?}: {}",
                    file,
                    dest_file,
                    copy.err().unwrap()
                );
            }
        }
    });
}

fn get_all_files(src: &PathBuf) -> Vec<PathBuf> {
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
        } else {
            files.push(file.path());
        }
    }

    files
}

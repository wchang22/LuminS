use std::fs;
use std::path::PathBuf;
use rayon::prelude::*;

const NUM_THREADS: usize = 7;

pub fn synchronize(src: &String, dest: &String) {
    let files = get_all_files(src);

    copy_files(&files, src, dest, NUM_THREADS);
}

fn copy_files(files: &Vec<PathBuf>, src: &String, dest: &String, num_threads: usize) {
    if files.len() <= num_threads {
        files.par_iter().for_each(|file| {
            let mut dest_file = PathBuf::new();
            dest_file.push(&dest);
            dest_file.push(file.strip_prefix(&src).unwrap());

            fs::create_dir_all(dest_file.parent().unwrap()).unwrap();
            fs::copy(file, dest_file).unwrap();
        });
    } else {
        files.par_chunks(files.len() / num_threads).for_each(|slice| {
            for file in slice.iter() {
                let mut dest_file = PathBuf::new();
                dest_file.push(&dest);
                dest_file.push(file.strip_prefix(&src).unwrap());
                fs::create_dir_all(dest_file.parent().unwrap()).unwrap();
                fs::copy(file, dest_file).unwrap();
            }
        });
    }
}

fn get_all_files(src: &String) -> Vec<PathBuf> {
    let dir = fs::read_dir(src);
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
        let metadata = fs::metadata(file.path());

        if metadata.is_err() {
            eprintln!("{:?} {}", file.path(), metadata.err().unwrap());
            continue;
        }

        let metadata = metadata.unwrap();

        if metadata.is_dir() {
            let path = file.path().into_os_string().into_string().unwrap();
            files.extend(get_all_files(&path));
        } else {
            files.push(file.path());
        }
    }

    files
}
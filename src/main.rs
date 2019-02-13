use std::env;
use std::fs;

mod lumins;
use lumins::core;

fn main() {
    let mut args = env::args();

    if args.len() != 3 {
        eprintln!("Usage: lumins [OPTION]... SOURCE... DESTINATION");
        return;
    }

    args.next();
    let src = args.next().unwrap();
    let dest = args.next().unwrap();

    let src_metadata = fs::metadata(&src);
    match src_metadata {
        Ok(m) => {
            if !m.is_dir() {
                eprintln!("Source Error: {} is not a directory", &src);
                return;
            }
        }
        Err(e) => {
            eprintln!("Source Error: {}", e);
            return;
        }
    };

    let dest_metadata = fs::metadata(&dest);
    match dest_metadata {
        Ok(m) => {
            if !m.is_dir() {
                eprintln!("Destination Error: {} is not a directory", &src);
                return;
            }
        }
        Err(e) => {
            eprintln!("Destination Error: {}", e);
            return;
        }
    };

    core::synchronize(src, dest);
}

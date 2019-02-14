use std::env;
use std::fs;

pub fn parse_args() -> Result<(String, String), ()> {
    let mut args = env::args();

    if args.len() != 3 {
        println!("Usage: lumins [OPTION]... SOURCE... DESTINATION");
        return Err(());
    }

    args.next();
    let src = args.next().unwrap();
    let dest = args.next().unwrap();

    let src_metadata = fs::metadata(&src);
    match src_metadata {
        Ok(m) => {
            if !m.is_dir() {
                eprintln!("Source Error: {} is not a directory", &src);
                return Err(());
            }
        }
        Err(e) => {
            eprintln!("Source Error: {}", e);
            return Err(());
        }
    };

    let create_dest = fs::create_dir_all(&dest);

    if create_dest.is_err() {
        eprintln!("Destination Error: {}", create_dest.err().unwrap());
        return Err(());
    }

    Ok((src, dest))
}

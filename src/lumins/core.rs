use std::fs;
use std::path::PathBuf;

pub fn synchronize(src: &String, dest: &String) {
    let dir = fs::read_dir(src);
    let dir = match dir {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    for file in dir {
        if file.is_err() {
            eprintln!("{}", file.err().unwrap());
            continue;
        }

        let file = file.unwrap();
        let metadata = fs::metadata(file.path());

        if metadata.is_err() {
            eprintln!("{}", metadata.err().unwrap());
            continue;
        }

        let metadata = metadata.unwrap();

        if metadata.is_dir() {
            let path = file.path().into_os_string().into_string().unwrap();
            let mut new_dest = String::new();
            new_dest.push_str(&dest);
            new_dest.push_str("/");
            new_dest.push_str(file.path().strip_prefix(&src).unwrap().to_str().unwrap());

            fs::create_dir_all(&new_dest).unwrap();

            synchronize(&path, &new_dest);
        } else {
            let mut dest_file = PathBuf::new();
            dest_file.push(&dest);
            dest_file.push(file.path().file_name().unwrap());
            fs::copy(file.path(), dest_file).unwrap();
        }
    }
}
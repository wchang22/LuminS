use std::fs;

/// Parses command line arguments for source and destination folders and
/// creates the destination folder if it does not exist
///
/// # Errors
/// This function will return an error in the following situations,
/// but is not limited to just these cases:
/// * `args` do not contain source and destination folders
/// * The source folder is not a valid directory
/// * The destination folder could not be created
pub fn parse_args(args: &[String]) -> Result<(String, String), ()> {
    if args.len() != 3 {
        println!("Usage: lumins SOURCE... DESTINATION");
        return Err(());
    }

    let src = &args[1];
    let dest = &args[2];

    // Check if src is valid
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

    // Create destination folder if not already existing
    let create_dest = fs::create_dir_all(&dest);
    if create_dest.is_err() {
        eprintln!("Destination Error: {}", create_dest.err().unwrap());
        return Err(());
    }

    Ok((src.to_string(), dest.to_string()))
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;

    #[test]
    fn without_args() {
        let args = vec![String::from("lumins")];
        assert_eq!(parse_args(&args), Err(()));
    }

    #[test]
    fn too_many_args() {
        let args = vec![
            String::from("lumins"),
            String::from("src"),
            String::from("dest"),
            String::from("extra"),
        ];
        assert_eq!(parse_args(&args), Err(()));
    }

    #[test]
    fn no_dest() {
        let args = vec![String::from("lumins"), String::from("src")];
        assert_eq!(parse_args(&args), Err(()));
    }

    #[test]
    fn invalid_src() {
        let args = vec![
            String::from("lumins"),
            String::from("/?"),
            String::from("dest"),
        ];
        assert_eq!(parse_args(&args), Err(()));
    }

    #[test]
    fn src_not_dir() {
        let args = vec![
            String::from("lumins"),
            String::from("./Cargo.toml"),
            String::from("dest"),
        ];
        assert_eq!(parse_args(&args), Err(()));
    }

    #[test]
    fn fail_create_dest() {
        let args = vec![
            String::from("lumins"),
            String::from("."),
            String::from("/asdf"),
        ];
        assert_eq!(parse_args(&args), Err(()));
    }

    #[test]
    fn parse_success() {
        let args = vec![
            String::from("lumins"),
            String::from("src"),
            String::from("test_dest"),
        ];
        assert_eq!(
            parse_args(&args),
            Ok((String::from("src"), String::from("test_dest")))
        );

        let test_dest = fs::read_dir("test_dest");
        assert_eq!(test_dest.is_ok(), true);

        fs::remove_dir("test_dest").unwrap();
    }
}

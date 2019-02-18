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
pub fn parse_args(src: &str, dest: &str) -> Result<(), ()> {
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

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;

    #[test]
    fn invalid_src() {
        let src = "/?";
        let dest = "/";
        assert_eq!(parse_args(src, dest), Err(()));
    }

    #[test]
    fn src_not_dir() {
        let src = "./Cargo.toml";
        let dest = "/";
        assert_eq!(parse_args(src, dest), Err(()));
    }

    #[test]
    fn fail_create_dest() {
        let src = ".";
        let dest = "/asdf";
        assert_eq!(parse_args(src, dest), Err(()));
    }

    #[test]
    fn parse_success() {
        const TEST_SRC: &str = "./src";
        const TEST_DIR: &str = "parse_success";

        assert_eq!(
            parse_args(TEST_SRC, TEST_DIR),
            Ok(())
        );

        let test_dest = fs::read_dir(TEST_DIR);
        assert_eq!(test_dest.is_ok(), true);

        fs::remove_dir(TEST_DIR).unwrap();
    }
}

use std::fs;

use clap::ArgMatches;

/// Struct to represent command line flags
#[repr(u32)]
pub enum Flag {
    Copy = 1 << 0,
    NoDelete = 1 << 1,
    Secure = 1 << 2,
    Verbose = 1 << 3,
}

/// Struct to represent the result of parsing args
pub struct ParseResult<'a> {
    pub src: &'a str,
    pub dest: &'a str,
    pub flags: u32,
}

/// Parses command line arguments for source and destination folders and
/// creates the destination folder if it does not exist
///
/// # Errors
/// This function will return an error in the following situations,
/// but is not limited to just these cases:
/// * The source folder is not a valid directory
/// * The destination folder could not be created
pub fn parse_args<'a>(args: &'a ArgMatches) -> Result<ParseResult<'a>, ()> {
    // Safe to unwrap since these are required
    let src = args.value_of("SOURCE").unwrap();
    let dest = args.value_of("DESTINATION").unwrap();

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

    // Parse for flags
    let mut flags = 0;
    if args.is_present("copy") {
        flags |= Flag::Copy as u32;
    }
    if args.is_present("verbose") {
        flags |= Flag::Verbose as u32;
    }
    if args.is_present("nodelete") {
        flags |= Flag::NoDelete as u32;
    }
    if args.is_present("secure") {
        flags |= Flag::Secure as u32;
    }

    Ok(ParseResult { src, dest, flags })
}

/// Determines whether or not a given flag is set in the given bitfield
///
/// # Arguments
/// * `bitfield`: a bitfield where each bit is a `Flag`
/// * `flag`: flag to query
///
/// # Returns
/// * true if `flag` is set in `bitfield`
/// * false if `flag` is not set in `bitfield`
pub fn contains_flag(bitfield: u32, flag: Flag) -> bool {
    (bitfield >> (((flag as u32) as f32).log2() as u32) & 1) == 1
}

#[cfg(test)]
mod test_contains_flag {
    use super::*;

    #[test]
    fn all_flags() {
        let flags = 0xFFFFFFFF;
        assert_eq!(contains_flag(flags, Flag::Copy), true);
        assert_eq!(contains_flag(flags, Flag::NoDelete), true);
        assert_eq!(contains_flag(flags, Flag::Secure), true);
        assert_eq!(contains_flag(flags, Flag::Verbose), true);
    }

    #[test]
    fn no_flags() {
        let flags = 0;
        assert_eq!(contains_flag(flags, Flag::Copy), false);
        assert_eq!(contains_flag(flags, Flag::NoDelete), false);
        assert_eq!(contains_flag(flags, Flag::Secure), false);
        assert_eq!(contains_flag(flags, Flag::Verbose), false);
    }

    #[test]
    fn some_flags() {
        let flags = 0b1010;
        assert_eq!(contains_flag(flags, Flag::Copy), false);
        assert_eq!(contains_flag(flags, Flag::NoDelete), true);
        assert_eq!(contains_flag(flags, Flag::Secure), false);
        assert_eq!(contains_flag(flags, Flag::Verbose), true);
    }
}

#[cfg(test)]
mod test {
    //    use super::*;
    //    use std::fs;
    //
    //    #[test]
    //    fn invalid_src() {
    //        let src = "/?";
    //        let dest = "/";
    //        assert_eq!(parse_args(src, dest), Err(()));
    //    }
    //
    //    #[test]
    //    fn src_not_dir() {
    //        let src = "./Cargo.toml";
    //        let dest = "/";
    //        assert_eq!(parse_args(src, dest), Err(()));
    //    }
    //
    //    #[test]
    //    fn fail_create_dest() {
    //        let src = ".";
    //        let dest = "/asdf";
    //        assert_eq!(parse_args(src, dest), Err(()));
    //    }
    //
    //    #[test]
    //    fn parse_success() {
    //        const TEST_SRC: &str = "./src";
    //        const TEST_DIR: &str = "parse_success";
    //
    //        assert_eq!(parse_args(TEST_SRC, TEST_DIR), Ok(()));
    //
    //        let test_dest = fs::read_dir(TEST_DIR);
    //        assert_eq!(test_dest.is_ok(), true);
    //
    //        fs::remove_dir(TEST_DIR).unwrap();
    //    }
}

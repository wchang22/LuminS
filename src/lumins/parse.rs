use std::fs;

use clap::ArgMatches;
use rayon_hash::HashSet;

/// Enum to represent command line flags
#[derive(Hash, Eq, PartialEq, Clone)]
#[repr(u8)]
pub enum Flag {
    Copy,
    NoDelete,
    Secure,
    Verbose,
    Sequential,
}

/// Struct to represent the result of parsing args
pub struct ParseResult<'a> {
    pub src: &'a str,
    pub dest: &'a str,
    pub flags: HashSet<Flag>,
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
    match fs::metadata(&src) {
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
    if let Err(e) = fs::create_dir_all(&dest) {
        eprintln!("Destination Error: {}", e);
        return Err(());
    }

    // Parse for flags
    let mut flags = HashSet::new();
    if args.is_present("copy") {
        flags.insert(Flag::Copy);
    }
    if args.is_present("verbose") {
        flags.insert(Flag::Verbose);
    }
    if args.is_present("nodelete") {
        flags.insert(Flag::NoDelete);
    }
    if args.is_present("secure") {
        flags.insert(Flag::Secure);
    }
    if args.is_present("sequential") {
        flags.insert(Flag::Sequential);
    }

    Ok(ParseResult { src, dest, flags })
}

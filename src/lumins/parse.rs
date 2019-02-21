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

/// Enum to represent subcommand type
#[derive(Eq, PartialEq, Clone)]
pub enum SubCommandType {
    Copy,
    Synchronize,
    Delete,
}

/// Struct to represent subcommands
pub struct SubCommand<'a> {
    pub src: Option<&'a str>,
    pub dest: &'a str,
    pub sub_command_type: SubCommandType,
}

/// Struct to represent the result of parsing args
pub struct ParseResult<'a> {
    pub sub_command: SubCommand<'a>,
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
    // These are safe to unwrap since subcommands are required
    let sub_command_name = args.subcommand_name().unwrap();
    let args = args.subcommand_matches(sub_command_name).unwrap();

    // These values are safe to unwrap since the args are required
    let sub_command = match sub_command_name {
        "copy" => SubCommand {
            src: Some(args.value_of("SOURCE").unwrap()),
            dest: args.value_of("DESTINATION").unwrap(),
            sub_command_type: SubCommandType::Copy,
        },
        "del" => SubCommand {
            src: None,
            dest: args.value_of("TARGET").unwrap(),
            sub_command_type: SubCommandType::Delete,
        },
        "sync" => SubCommand {
            src: Some(args.value_of("SOURCE").unwrap()),
            dest: args.value_of("DESTINATION").unwrap(),
            sub_command_type: SubCommandType::Synchronize,
        },
        _ => return Err(()),
    };

    // Validate directories
    match sub_command.sub_command_type {
        SubCommandType::Delete => {
            // Target directory must be a valid directory
            match fs::metadata(sub_command.dest) {
                Ok(m) => {
                    if !m.is_dir() {
                        eprintln!("Target Error: {} is not a directory", sub_command.dest);
                        return Err(());
                    }
                }
                Err(e) => {
                    eprintln!("Target Error: {}", e);
                    return Err(());
                }
            };
        }
        SubCommandType::Copy | SubCommandType::Synchronize => {
            // Check if src is valid
            match fs::metadata(sub_command.src.unwrap()) {
                Ok(m) => {
                    if !m.is_dir() {
                        eprintln!(
                            "Source Error: {} is not a directory",
                            sub_command.src.unwrap()
                        );
                        return Err(());
                    }
                }
                Err(e) => {
                    eprintln!("Source Error: {}", e);
                    return Err(());
                }
            };

            // Create destination folder if not already existing
            if let Err(e) = fs::create_dir_all(sub_command.dest) {
                eprintln!("Destination Error: {}", e);
                return Err(());
            }
        }
    }

    static FLAG_NAMES: [&str; 5] = ["copy", "verbose", "nodelete", "secure", "sequential"];
    static FLAGS: [Flag; 5] = [
        Flag::Copy,
        Flag::Verbose,
        Flag::NoDelete,
        Flag::Secure,
        Flag::Sequential,
    ];

    // Parse for flags
    let mut flags = HashSet::new();
    for (i, &flag_name) in FLAG_NAMES.iter().enumerate() {
        if args.is_present(flag_name) {
            flags.insert(FLAGS[i].clone());
        }
    }

    Ok(ParseResult { sub_command, flags })
}

//! Some utilities for command line parsing.

use std::fs;
use std::path::PathBuf;

use clap::ArgMatches;
use rayon_hash::HashSet;

/// Enum to represent command line flags
#[derive(Hash, Eq, PartialEq, Clone)]
#[repr(u8)]
pub enum Flag {
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
    Remove,
}

/// Struct to represent subcommands
pub struct SubCommand<'a> {
    pub src: Option<&'a str>,
    pub dest: Vec<String>,
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

    const FLAG_NAMES: [&str; 4] = ["verbose", "nodelete", "secure", "sequential"];
    const FLAGS: [Flag; 4] = [
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

    // These values are safe to unwrap since the args are required
    let mut sub_command = match sub_command_name {
        "cp" => SubCommand {
            src: Some(args.value_of("SOURCE").unwrap()),
            dest: vec![args.value_of("DESTINATION").unwrap().to_string()],
            sub_command_type: SubCommandType::Copy,
        },
        "rm" => SubCommand {
            src: None,
            dest: args
                .values_of("TARGET")
                .unwrap()
                .map(|value| value.to_string())
                .collect(),
            sub_command_type: SubCommandType::Remove,
        },
        "sync" => SubCommand {
            src: Some(args.value_of("SOURCE").unwrap()),
            dest: vec![args.value_of("DESTINATION").unwrap().to_string()],
            sub_command_type: SubCommandType::Synchronize,
        },
        _ => return Err(()),
    };

    // Validate directories
    match sub_command.sub_command_type {
        SubCommandType::Remove => {
            sub_command.dest.retain(|dest| {
                // Target directory must be a valid directory
                match fs::metadata(dest) {
                    Ok(m) => {
                        if !m.is_dir() {
                            eprintln!("Target Error -- {} is not a directory", dest);
                        }
                        m.is_dir()
                    }
                    Err(e) => {
                        eprintln!("Target Error -- {}: {}", dest, e);
                        false
                    }
                }
            });

            if sub_command.dest.is_empty() {
                return Err(());
            }
        }
        SubCommandType::Copy | SubCommandType::Synchronize => {
            // Check if src is valid
            match fs::metadata(sub_command.src.unwrap()) {
                Ok(m) => {
                    if !m.is_dir() {
                        eprintln!(
                            "Source Error -- {} is not a directory",
                            sub_command.src.unwrap()
                        );
                        return Err(());
                    }
                }
                Err(e) => {
                    eprintln!("Source Error -- {}: {}", sub_command.src.unwrap(), e);
                    return Err(());
                }
            };

            // If the directory already exists, then the directory is directory + src name
            if sub_command.sub_command_type == SubCommandType::Copy
                && fs::metadata(&sub_command.dest[0]).is_ok()
            {
                let mut new_dest = PathBuf::from(&sub_command.dest[0]);
                let src_name = PathBuf::from(sub_command.src.unwrap());
                if let Some(src_name) = src_name.file_name() {
                    new_dest.push(src_name);
                    sub_command.dest = vec![new_dest.to_string_lossy().to_string()];
                }
            }

            if fs::metadata(&sub_command.dest[0]).is_err() {
                // Create destination folder if not already existing
                match fs::create_dir_all(&sub_command.dest[0]) {
                    Ok(_) => {
                        if flags.contains(&Flag::Verbose) {
                            println!("Creating dir {:?}", sub_command.dest[0]);
                        }
                    }
                    Err(e) => {
                        eprintln!("Destination Error -- {}: {}", sub_command.dest[0], e);
                        return Err(());
                    }
                }
            }
        }
    }

    Ok(ParseResult { sub_command, flags })
}

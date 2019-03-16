use std::env;
use std::process;

use clap::{load_yaml, App};
use env_logger::Builder;
use log::{error, LevelFilter};

use lms::core;
use lms::parse::{self, Flag, SubCommandType};
use lms::PROGRESS_BAR;

fn main() {
    // Parse command args
    let yaml = load_yaml!("cli.yml");
    let args = App::from_yaml(yaml).get_matches();

    // Determine subcommands and flags from args
    let (sub_command, flags) = match parse::parse_args(&args) {
        Ok(f) => (f.sub_command, f.flags),
        Err(_) => process::exit(1),
    };

    // If verbose, enable logging
    if flags.contains(&Flag::Verbose) {
        env::set_var("RUST_LOG", "info");
        Builder::new()
            .format(|_, record| {
                PROGRESS_BAR.println(format!("{}", record.args()));
                Ok(())
            })
            .filter(None, LevelFilter::Info)
            .init();
    }

    // Call correct core function depending on subcommand
    let result = match sub_command.sub_command_type {
        SubCommandType::Copy => core::copy(sub_command.src.unwrap(), &sub_command.dest[0], flags),
        SubCommandType::Remove => sub_command
            .dest
            .iter()
            .map(|dest| core::remove(dest, flags.clone()))
            .collect(),
        SubCommandType::Synchronize => {
            core::synchronize(sub_command.src.unwrap(), &sub_command.dest[0], flags)
        }
    };

    // End and remove the progress bar
    PROGRESS_BAR.finish_and_clear();

    // If error, print to stderr and exit
    if let Err(e) = result {
        eprintln!("{}", e);
        process::exit(1);
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test_main {
    use std::fs;
    use std::process::Command;

    #[test]
    fn test_no_args() {
        Command::new("cargo")
            .args(&["build", "--release"])
            .output()
            .unwrap();

        let output = Command::new("target/release/lms").output().unwrap();

        assert_eq!(output.status.success(), false);
    }

    #[test]
    fn test_no_dest() {
        Command::new("cargo")
            .args(&["build", "--release"])
            .output()
            .unwrap();

        let output = Command::new("target/release/lms")
            .args(&["sync", "src"])
            .output()
            .unwrap();

        assert_eq!(output.status.success(), false);
    }

    #[test]
    fn test_too_many_args() {
        Command::new("cargo")
            .args(&["build", "--release"])
            .output()
            .unwrap();

        let output = Command::new("target/release/lms")
            .args(&["sync", "src", "dest", "dest"])
            .output()
            .unwrap();

        assert_eq!(output.status.success(), false);
    }

    #[test]
    fn test_invalid_args() {
        Command::new("cargo")
            .args(&["build", "--release"])
            .output()
            .unwrap();

        let output = Command::new("target/release/lms")
            .args(&["sync", "a", "dest"])
            .output()
            .unwrap();

        assert_eq!(output.status.success(), false);
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn test_copy() {
        Command::new("cargo")
            .args(&["build", "--release"])
            .output()
            .unwrap();

        const TEST_SOURCE: &str = "target/debug";
        const TEST_DEST: &str = "test_main_test_copy";

        Command::new("target/release/lms")
            .args(&["cp", "-v", TEST_SOURCE, TEST_DEST])
            .output()
            .unwrap();

        let diff = Command::new("diff")
            .args(&["-r", TEST_SOURCE, TEST_DEST])
            .output()
            .unwrap();

        assert_eq!(diff.status.success(), true);

        fs::remove_dir_all(TEST_DEST).unwrap();
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn test_secure() {
        Command::new("cargo")
            .args(&["build", "--release"])
            .output()
            .unwrap();

        const TEST_SOURCE: &str = "target/debug";
        const TEST_DEST: &str = "test_main_test_secure";
        fs::create_dir_all(TEST_DEST).unwrap();

        Command::new("target/release/lms")
            .args(&["sync", "-s", TEST_SOURCE, TEST_DEST])
            .output()
            .unwrap();

        let diff = Command::new("diff")
            .args(&["-r", TEST_SOURCE, TEST_DEST])
            .output()
            .unwrap();

        assert_eq!(diff.status.success(), true);

        fs::remove_dir_all(TEST_DEST).unwrap();
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn test_sequential() {
        Command::new("cargo")
            .args(&["build", "--release"])
            .output()
            .unwrap();

        const TEST_SOURCE: &str = "target/debug";
        const TEST_DEST: &str = "test_main_test_sequential";

        Command::new("target/release/lms")
            .args(&["sync", "-S", TEST_SOURCE, TEST_DEST])
            .output()
            .unwrap();

        let diff = Command::new("diff")
            .args(&["-r", TEST_SOURCE, TEST_DEST])
            .output()
            .unwrap();

        assert_eq!(diff.status.success(), true);

        fs::remove_dir_all(TEST_DEST).unwrap();
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn test_sequential_copy() {
        Command::new("cargo")
            .args(&["build", "--release"])
            .output()
            .unwrap();

        const TEST_SOURCE: &str = "target/debug";
        const TEST_DEST: &str = "test_main_test_sequential_copy";

        Command::new("target/release/lms")
            .args(&["cp", "-S", TEST_SOURCE, TEST_DEST])
            .output()
            .unwrap();

        let diff = Command::new("diff")
            .args(&["-r", TEST_SOURCE, TEST_DEST])
            .output()
            .unwrap();

        assert_eq!(diff.status.success(), true);

        fs::remove_dir_all(TEST_DEST).unwrap();
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn test_no_delete() {
        Command::new("cargo")
            .args(&["build", "--release"])
            .output()
            .unwrap();

        const TEST_SOURCE1: &str = "test_main_test_no_delete_source1";
        const TEST_SOURCE2: &str = "test_main_test_no_delete_source2";
        const TEST_DEST: &str = "test_main_test_no_delete_out";
        const TEST_EXPECTED: &str = "test_main_test_no_delete_expected";
        const TEST_FILE1: &str = "Cargo.toml";
        const TEST_FILE2: &str = "Cargo.lock";

        fs::create_dir_all(TEST_SOURCE1).unwrap();
        fs::create_dir_all(TEST_SOURCE2).unwrap();
        fs::create_dir_all(TEST_EXPECTED).unwrap();

        fs::copy(TEST_FILE1, [TEST_SOURCE1, TEST_FILE1].join("/")).unwrap();
        fs::copy(TEST_FILE2, [TEST_SOURCE2, TEST_FILE2].join("/")).unwrap();
        fs::copy(TEST_FILE1, [TEST_EXPECTED, TEST_FILE1].join("/")).unwrap();
        fs::copy(TEST_FILE2, [TEST_EXPECTED, TEST_FILE2].join("/")).unwrap();

        Command::new("target/release/lms")
            .args(&["cp", TEST_SOURCE1, TEST_DEST])
            .output()
            .unwrap();

        Command::new("target/release/lms")
            .args(&["sync", "-n", TEST_SOURCE2, TEST_DEST])
            .output()
            .unwrap();

        let diff = Command::new("diff")
            .args(&["-r", TEST_DEST, TEST_EXPECTED])
            .output()
            .unwrap();

        assert_eq!(diff.status.success(), true);

        fs::remove_dir_all(TEST_SOURCE1).unwrap();
        fs::remove_dir_all(TEST_SOURCE2).unwrap();
        fs::remove_dir_all(TEST_DEST).unwrap();
        fs::remove_dir_all(TEST_EXPECTED).unwrap();
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn test_remove() {
        Command::new("cargo")
            .args(&["build", "--release"])
            .output()
            .unwrap();

        const TEST_SOURCE: &str = "target/debug";
        const TEST_DEST: &str = "test_main_test_remove";
        fs::create_dir_all(TEST_DEST).unwrap();

        Command::new("cp")
            .args(&["-r", TEST_SOURCE, TEST_DEST])
            .output()
            .unwrap();

        Command::new("target/release/lms")
            .args(&["rm", TEST_DEST])
            .output()
            .unwrap();

        assert_eq!(fs::read_dir(TEST_DEST).is_err(), true);
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn test_remove_multiple() {
        Command::new("cargo")
            .args(&["build", "--release"])
            .output()
            .unwrap();

        const TEST_SOURCE: &str = "target/debug";
        const TEST_DEST: [&str; 2] = ["test_main_test_remove1", "test_main_test_remove2"];
        fs::create_dir_all(TEST_DEST[0]).unwrap();
        fs::create_dir_all(TEST_DEST[1]).unwrap();

        Command::new("cp")
            .args(&["-r", TEST_SOURCE, TEST_DEST[0]])
            .output()
            .unwrap();

        Command::new("cp")
            .args(&["-r", TEST_SOURCE, TEST_DEST[1]])
            .output()
            .unwrap();

        Command::new("target/release/lms")
            .args(&["rm", TEST_DEST[0], TEST_DEST[1]])
            .output()
            .unwrap();

        assert_eq!(fs::read_dir(TEST_DEST[0]).is_err(), true);
        assert_eq!(fs::read_dir(TEST_DEST[1]).is_err(), true);
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn test_sequential_remove() {
        Command::new("cargo")
            .args(&["build", "--release"])
            .output()
            .unwrap();

        const TEST_SOURCE: &str = "target/debug";
        const TEST_DEST: &str = "test_main_test_sequential_remove";
        fs::create_dir_all(TEST_DEST).unwrap();

        Command::new("cp")
            .args(&["-r", TEST_SOURCE, TEST_DEST])
            .output()
            .unwrap();

        Command::new("target/release/lms")
            .args(&["rm", "-S", TEST_DEST])
            .output()
            .unwrap();

        assert_eq!(fs::read_dir(TEST_DEST).is_err(), true);
    }
}

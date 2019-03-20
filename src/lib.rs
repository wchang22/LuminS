//! LuminS (lms) is a fast and reliable alternative to rsync for synchronizing local files
//!
//! ```usage
//! USAGE:
//!    lms [SUBCOMMAND]
//!
//! FLAGS:
//!    -h, --help       Prints help information
//!    -V, --version    Prints version information
//!
//! SUBCOMMANDS:
//!    cp      Multithreaded directory copy
//!    help    Prints this message or the help of the given subcommand(s)
//!    rm      Multithreaded directory remove
//!    sync    Multithreaded directory synchronization [aliases: s]
//! ```

mod lumins;
pub use lumins::*;

//! LuminS (lms) is a fast and reliable alternative to rsync for synchronizing local files
//!
//! ```ignore
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

use indicatif::{ProgressBar, ProgressStyle};
use lazy_static::lazy_static;

lazy_static! {
    // Create a progress bar for operations
    pub static ref PROGRESS_BAR: ProgressBar = {
        let pb = ProgressBar::new(0);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{bar:40.green/white}] {pos}/{len} ({eta})"),
        );
        pb
    };
}

//! Keeps track of LuminS' progress

use indicatif::{ProgressBar, ProgressStyle};
use lazy_static::lazy_static;

lazy_static! {
    /// Provides a bar that shows the number of files
    /// copied, synchronized, or deleted, out of the total number of files
    pub static ref PROGRESS_BAR: ProgressBar = {
        let progress_bar = ProgressBar::new(0);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{bar:40.green/blue}] {pos}/{len} ({eta})"),
        );
        progress_bar
    };
}

/// Initializes PROGRESS_BAR with `length` and sets draw delta
/// # Arguments
/// * `length`: Length fo the bar to set
pub fn progress_init(length: u64) {
    PROGRESS_BAR.set_length(length);
    PROGRESS_BAR.set_draw_delta(length / 1000);
    PROGRESS_BAR.set_position(0);
}

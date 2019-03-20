//! Keeps track of LuminS' progress

use indicatif::{ProgressBar, ProgressStyle};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref PROGRESS_BAR: ProgressBar = {
        let progress_bar = ProgressBar::new(100000);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{bar:40.green/blue}] {pos}/{len} ({eta})"),
        );
        progress_bar.set_draw_delta(10);
        progress_bar
    };
}

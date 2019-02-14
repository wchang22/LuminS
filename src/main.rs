use std::env;

mod lumins;
pub use lumins::{core, parse};

fn main() {
    let args: Vec<String> = env::args().collect();

    let (src, dest) = match parse::parse_args(&args) {
        Ok((s, t)) => (s, t),
        Err(_) => return,
    };

    core::synchronize(&src, &dest);
}

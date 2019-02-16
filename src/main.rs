use std::env;

mod lumins;
pub use lumins::{core, file_ops, parse};

fn main() {
    let args: Vec<String> = env::args().collect();

    let (src, dest) = match parse::parse_args(&args) {
        Ok((s, t)) => (s, t),
        Err(_) => return,
    };

    let result = core::synchronize(&src, &dest);
    if result.is_err() {
        eprintln!("{}", result.err().unwrap());
    }
}

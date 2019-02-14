mod lumins;
use lumins::{core, parse};

fn main() {
    let (src, dest) = match parse::parse_args() {
        Ok((s, t)) => (s, t),
        Err(_) => return,
    };

    core::synchronize(&src, &dest);
}

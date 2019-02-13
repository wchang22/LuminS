mod lumins;
use lumins::{parse, core};

fn main() {
    let (src, dest) = match parse::parse_args() {
        Ok((s, t)) => (s, t),
        Err(_) => return,
    };

    core::synchronize(src, dest);
}

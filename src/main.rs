#[macro_use]
extern crate log;

use std::env;
use std::io::Write;

use clap::{load_yaml, App};
use env_logger::Builder;
use log::LevelFilter;

mod lumins;
pub use lumins::{core, file_ops, parse};

fn main() {
    let yaml = load_yaml!("cli.yml");
    let args = App::from_yaml(yaml).get_matches();

    let (src, dest, flags) = match parse::parse_args(&args) {
        Ok(f) => (f.src, f.dest, f.flags),
        Err(_) => return,
    };

    let result;

    if parse::contains_flag(flags, parse::Flag::Verbose) {
        env::set_var("RUST_LOG", "info");
        Builder::new()
            .format(|buf, record| writeln!(buf, "{}", record.args()))
            .filter(None, LevelFilter::Info)
            .init();
    }

    if parse::contains_flag(flags, parse::Flag::Copy) {
        result = core::copy(src, dest);
    } else {
        result = core::synchronize(src, dest, flags);
    }

    if result.is_err() {
        eprintln!("{}", result.err().unwrap());
    }
}

#[macro_use]
extern crate clap;
use clap::App;

mod lumins;
pub use lumins::{core, file_ops, parse};

fn main() {
    let yaml = load_yaml!("cli.yml");
    let args = App::from_yaml(yaml).get_matches();

    let src = args.value_of("SOURCE").unwrap();
    let dest = args.value_of("DESTINATION").unwrap();

    if parse::parse_args(src, dest).is_err() {
        return;
    }

    let result = core::synchronize(&src, &dest);
    if result.is_err() {
        eprintln!("{}", result.err().unwrap());
    }
}

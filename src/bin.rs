use valhallac;
use std::env;

fn is_vh_file(filename : &String) -> bool {
    filename.ends_with(".vh")
}

pub fn main() {
    let args = env::args();

    let files = args.filter(is_vh_file);

    for file in files {
        valhallac::parse(&file);
    }
}
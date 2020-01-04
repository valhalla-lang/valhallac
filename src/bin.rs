use valhallac;
use std::env;

fn is_vh_file(filename : &String) -> bool {
    filename.ends_with(".vh")
}

pub fn main() -> Result<(), i32> {
    let args = env::args();

    let files = args.filter(is_vh_file);

    for file in files {
        let root = valhallac::parse(&file);
        let block = valhallac::compile(&root);

        let out = file[..file.len() - 3].to_owned();
        valhallac::binary_gen(&block, out);
    }
    Ok(())
}
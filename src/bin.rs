use valhallac;

use std::env;
use std::fs::File;
use std::io::Write;

fn is_vh_file(filename : &String) -> bool {
    filename.ends_with(".vh")
}

pub fn main() -> Result<(), i32> {
    let args = env::args();

    let files = args.filter(is_vh_file);

    for file in files {
        // Parse source into tree.
        let root = valhallac::parse(&file);
        // Then compile into series of instructions,
        //   stored as a code block.
        let block = valhallac::compile(&root);

        // Pick name of outfile.
        let out = file[..file.len() - 3].to_owned();
        // Convert code block to byte-stream, which will be
        //   the file's contents.
        let bytes = valhallac::binary_blob(&block);

        // Write blob to file.
        let mut file = File::create(out).expect("Could not create binary.");
        file.write(&bytes).expect("Could not write to binary.");
    }
    Ok(())
}

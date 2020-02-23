use valhallac;

use std::env;
use std::{fs::File, path::Path};
use std::io::Write;

fn is_vh_file(filename : &String) -> bool {
    filename.ends_with(".vh") && Path::new(filename).exists()
}

pub fn main() -> Result<(), i32> {
    let mut args = env::args();
    args.next();

    let mut files = args.filter(is_vh_file).peekable();

    if files.peek().is_none() {
        println!("No valid input file given.");
        std::process::exit(1);
    }

    for file in files {
        #[cfg(not(feature="debug"))]
        println!("Compiling `{}`...", file);
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
        let mut file = File::create(&out).expect("Could not create binary.");
        file.write_all(&bytes).expect("Could not write to binary.");

        #[cfg(not(feature="debug"))]
        println!("Binary written to `{}`.", out);
    }
    Ok(())
}

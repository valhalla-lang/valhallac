use ::valhallac;

use std::env;
use std::{fs::File, path::Path};
use std::io::Write;
use std::time::Instant;
use std::collections::HashMap;

use lazy_static::lazy_static;

use colored::*;

fn is_vh_file(filename : &String) -> bool {
    filename.ends_with(".vh")
    && Path::new(filename).exists()
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Flags {
    Verbose, Out,
    Version
}

// TODO: Halt on unrecognised options.
/// Collect flags and options passed to the executable.
fn collect_flags() -> HashMap<Flags, String> {
    let mut map = HashMap::new();
    let dummy = String::new();

    let mut maybe_argument : Option<Flags> = None;

    for arg in env::args() {
        // Collect any chars after a dash (e.g. `-v` for verbose)
        // and build a set of flags from the specific letters.
        let arg_str = arg.to_string();

        if let Some(argument) = maybe_argument {
            // Key assuredly exists.
            *map.get_mut(&argument).unwrap() = arg_str;
            maybe_argument = None;
            continue;  // Not an option, but an argument.
        }

        let mut singleton = |flag: Flags| map.insert(flag, dummy.clone());

        if arg_str.starts_with("--") {
            let name = arg_str.get(2..);
            match name {
                Some("verbose") => singleton(Flags::Verbose),
                Some("version") => singleton(Flags::Version),
                Some("out") => {
                    maybe_argument = Some(Flags::Out);
                    singleton(Flags::Out)
                },
                Some(&_) | None => None
            };
        } else if arg_str.starts_with('-') {
            let chars = arg_str.split("");
            for c in chars {
                if c == "-" { continue; }
                if c == "v" {
                    singleton(Flags::Verbose);
                } else if c == "o" {
                    maybe_argument = Some(Flags::Out);
                    singleton(Flags::Out);
                }
            }
        }
    }
    map
}

macro_rules! not_debug {
    ($verbose:expr, $block:block) => {
        #[cfg(not(feature="debug"))] {
            if $verbose $block
        }
    };
}

fn argument_error(msg : &str) {
    println!("{} {}", "[**]".red().bold(), msg.bold());
}

lazy_static! {
    static ref INFO : String = format!("{}", " :: ".bold().white());
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    valhallac::set_panic();

    let mut args = env::args();
    args.next();

    let flags = collect_flags();

    if flags.contains_key(&Flags::Version) {
        let (major, minor, tiny) = valhallac::VERSION;
        println!("(valhallac): v{}.{}.{}",
            major, minor, tiny);
        return Ok(());
    }

    #[allow(unused_variables)]
    let verbose : bool = flags.contains_key(&Flags::Verbose);

    #[allow(unused_variables)]
    #[cfg(feature="debug")]
    let verbose = true;

    let mut files = args.filter(is_vh_file).peekable();

    if files.peek().is_none() {
        argument_error("No valid input file given.");
        std::process::exit(1);
    }

    let begin = Instant::now();

    for file in files {
        not_debug!(verbose, {
                println!("{}{} `{}'...", *INFO,
                     "Parsing".bold().blue(),
                     file.underline().white());
        });
        // Parse source into tree.
        let root = valhallac::parse(&file);
        unsafe {
            if !valhallac::PANIC_MESSAGE.is_empty() {
                // An error in parsing, halt compilation.
                panic!("Parse error, will not compile bad tree.");
            }
        }

        // Then compile into series of instructions,
        //   stored as a code block.
        not_debug!(verbose, {
            println!("{}{}...", *INFO,
                     "Compiling".bold().blue());
        });
        let block = valhallac::compile(&root);

        // Pick name of outfile.
        let out = if let Some(out_location) = flags.get(&Flags::Out) {
            out_location.to_owned()
        } else {
            file[..file.len() - 3].to_owned() + ".out"
        };

        if out.is_empty() {
            argument_error("Empty/invalid output file specified.");
            std::process::exit(1);
        }

        // Convert code block to byte-stream, which will be
        //   the file's contents.
        let bytes = valhallac::binary_blob(&block);

        // Write blob to file.
        let mut file = File::create(&out)?;
        file.write_all(&bytes)?;

        not_debug!(verbose, {
            println!("{}{} to `{}'.", *INFO,
                     "Binary written".bold().blue(),
                     out.underline().white());
        });
    }


    #[allow(unused_variables)] {
        let elapsed = begin.elapsed();
        let seconds = elapsed.as_secs_f64();

        not_debug!(verbose, {
            print!("{}{} ", *INFO, "Took".bold().blue());
            println!("{}", if seconds < 0.1f64 {
                format!("{}ms.", begin.elapsed().as_millis())
            } else {
                format!("{:0.5}s", seconds)
            }.white());
        });
    }

    Ok(())
}

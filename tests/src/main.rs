#![feature(set_stdio)]

use valhallac;

use colored::*;

use std::{fs, path::Path, ffi::OsStr};
use std::panic;

type DynErr = Box<dyn std::error::Error>;

fn get_source(path : &Path) -> Result<String, ()> {
    let mut source = Err(());
    if path.is_file() && path.extension() == Some(OsStr::new("vh")) {
        source = fs::read_to_string(path).map_err(|_| ());
    }
    source
}

struct Ratio(u32, u32);

fn on_vh_files<F>(dir_path : &str, mut lambda : F) -> Result<Ratio, DynErr>
    where F : FnMut(&Path, String) -> bool {
    let (mut passes, mut total) = (0, 0);
    for entry in fs::read_dir(dir_path)? {
        total += 1;
        if let Ok(path) = entry {
            let path = path.path();
            if let Ok(source) = get_source(&path) {
               if lambda(&path, source) { passes += 1 }
            } else {  // Otherwise just skip.
                println!("Skipping `{}'...", path.to_string_lossy());
            }
        }
    }
    Ok(Ratio(passes, total))
}

use std::io::prelude::Write;

fn redirect_stderr(file : &str) -> Result<(), DynErr> {
    let f = fs::File::create(file)?;
    let sink : Box<dyn Send + Write> = Box::new(f);
    std::io::set_panic(Some(sink));
    Ok(())
}

fn status(good : bool) -> String {
    if good {
        "Ok".green()
    } else {
        "Failed".red()
    }.bold().to_string()
}

fn main() -> Result<(), DynErr> {
    valhallac::set_panic();
    redirect_stderr("stderr.log")?;

    let mut count = 0;
    let mut compile_attempt = |path: &Path, source: String| {
        count += 1;
        let filename = path.to_string_lossy();
        let prefix = format!("{: >4}. (`{}'):",
            count.to_string().bold(),
            path.file_stem().unwrap()
                .to_string_lossy()
                .underline()
                .white());
        // Catch errors:
        let did_panic = panic::catch_unwind(|| unsafe {
            let tree = valhallac::parse_source(&source, &filename);
            if valhallac::PANIC_MESSAGE.is_empty() {
                // Try to compile.
                valhallac::compile(&tree);
                if !valhallac::PANIC_MESSAGE.is_empty() {
                    panic!("Did not pass.");
                }
            } else {
                panic!("Did not pass.");
            }
        });
        print!("{} {} ", prefix, ".".repeat(80 - prefix.len()));
        did_panic.is_ok()
    };

    // Expecting success:
    println!("{} {}", "==>".blue().bold(),
        "Expecting compilation success:".white().bold());
    let succ_ratio = on_vh_files("./expect_success", |path, source| {
        let passed = compile_attempt(path, source);
        println!("{}", status(passed));
        passed
    })?;
    println!();

    // Expecting failure:
    println!("{} {}", "==>".blue().bold(),
        "Expecting compilation failure:".white().bold());
    let fail_ratio = on_vh_files("./expect_fail", |path, source| {
        let passed = compile_attempt(path, source);
        println!("{}", status(!passed));
        passed
    })?;
    println!();

    // Results:
    println!("{}", format!("Success Tests: {}/{}.",
        succ_ratio.0.to_string().yellow(),
        succ_ratio.1.to_string().yellow()).bold());
    println!("{}", format!("Failure Tests: {}/{}.",
        fail_ratio.0.to_string().yellow(),
        fail_ratio.1.to_string().yellow()).bold());

    Ok(())
}

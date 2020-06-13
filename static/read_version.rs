use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use toml;

pub fn out() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("version.rs");
    let mut f = File::create(&dest_path).unwrap();

    let mut config_f = File::open("Cargo.toml")?;
    let mut config_str = String::new();
    config_f.read_to_string(&mut config_str)?;

    let config: toml::Value = toml::from_str(&config_str)
        .unwrap();

    println!("{:#?}", config);

    match &config["package"]["version"] {
        toml::Value::String(version) => {
            if let &[major, minor, tiny] = version
                .split(".")
                .map(|s| s.parse::<u8>().unwrap())
                .collect::<Vec<_>>().as_slice() {

                f.write_all(format!("
                    const fn read_version() -> (u8, u8, u8) {{
                        return ({}, {}, {});
                    }}
                ", major, minor, tiny).as_bytes())?;
            } else {
                panic!("Version string should be three numbers \
                    separated by two dots.");
            }
        }
        _ => panic!("Version in `Config.toml' should be a string!")
    }

    Ok(())
}

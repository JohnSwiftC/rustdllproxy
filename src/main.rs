pub mod parsedllexports;

fn main() {
    let exports = parsedllexports::parse_dll_exports("../rustdll/shitty.dll").unwrap();
    println!("{:#?}", exports);
}

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn create_rust_lib_crate<P: AsRef<Path>>(
    dir_path: P, 
    crate_name: &str, 
    exports: Vec<&str>,
    dependencies: Option<Vec<&str>>
) -> io::Result<PathBuf> {
    let crate_dir = dir_path.as_ref().join(crate_name);
    
    fs::create_dir_all(&crate_dir)?;
    
    let src_dir = crate_dir.join("src");
    fs::create_dir(&src_dir)?;
    
    let lib_path = src_dir.join("lib.rs");
    File::create(lib_path)?;
    
    let cargo_path = crate_dir.join("Cargo.toml");
    let mut cargo_file = File::create(cargo_path)?;
    
    writeln!(cargo_file, "[package]")?;
    writeln!(cargo_file, "name = \"{}\"", crate_name)?;
    writeln!(cargo_file, "version = \"0.1.0\"")?;
    writeln!(cargo_file, "edition = \"2021\"")?;
    writeln!(cargo_file)?;
    
    writeln!(cargo_file, "[lib]")?;
    writeln!(cargo_file, "crate-type = [\"cdylib\"]")?;
    writeln!(cargo_file)?;
    
    writeln!(cargo_file, "[dependencies]")?;
    
    if let Some(deps) = dependencies {
        for dep in deps {
            writeln!(cargo_file, "{}", dep)?;
        }
    }
    
    // Create .cargo directory and config file for linker
    let cargo_config_dir = crate_dir.join(".cargo");
    fs::create_dir_all(&cargo_config_dir)?;
    
    let config_path = cargo_config_dir.join("config.toml");
    let mut config_file = File::create(config_path)?;
    
    writeln!(config_file, "[build]")?;
    
    #[cfg(target_os = "windows")]
    {
        writeln!(config_file, "rustflags = [\"-C\", \"link-args=/DEF:{}.def\"]", crate_name)?;
    }
    
    // Create .def file
    let def_path = crate_dir.join(format!("{}.def", crate_name));
    let mut def_file = File::create(def_path)?;
    
    writeln!(def_file, "LIBRARY {}", crate_name)?;
    writeln!(def_file, "EXPORTS")?;
    
    for export in exports {
        writeln!(def_file, "    {}", export)?;
    }
    
    Ok(crate_dir)
}
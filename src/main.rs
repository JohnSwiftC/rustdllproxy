pub mod parsedllexports;
use std::error::Error;
use std::collections::HashMap;

use clap::{Parser};

fn main() -> Result<(), Box<dyn Error>> {
    let cli = CLI::parse();
    let mut dlls_and_exports = HashMap::new();

    for dll_path in cli.dll_paths {
        let exports = parsedllexports::parse_dll_exports(&dll_path).expect("Bad DLL");
        let dll_name = dll_path.file_name().expect("Dll name not specified in file path");
        dlls_and_exports.insert(dll_name.to_str().expect("Invalid path encoding").to_owned(), exports);
    }
    
    let new_dir = cli.new_dir.unwrap_or(".".to_owned());
    let new_name = cli.new_name;

    let dependencies = vec!["dllproxymacros = \"0.1.0\"", "winapi = { version = \"0.3.9\", features = [\"libloaderapi\", \"minwindef\"] }"];

    create_rust_lib_crate(new_dir, &new_name, dlls_and_exports, Some(dependencies))?;

    Ok(())

}

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn create_rust_lib_crate<P: AsRef<Path>>(
    dir_path: P, 
    crate_name: &str,
    dlls_and_exports: HashMap<String, Vec<String>>,
    dependencies: Option<Vec<&str>>
) -> io::Result<PathBuf> {

    // Create src directory
    let crate_dir = dir_path.as_ref().join(crate_name);
    fs::create_dir_all(&crate_dir)?;
    
    // Create src directory
    let src_dir = crate_dir.join("src");
    fs::create_dir(&src_dir)?;
    
    // Create lib.rs
    let lib_path = src_dir.join("lib.rs");
    let mut lib_file = File::create(lib_path)?;
    
    // Create .def file
    let def_path = crate_dir.join(format!("{}.def", crate_name));
    let mut def_file = File::create(&def_path)?;
    
    // Create cargo.toml
    let cargo_path = crate_dir.join("Cargo.toml");
    let mut cargo_file = File::create(cargo_path)?;

    // Create .cargo directory
    let cargo_config_dir = crate_dir.join(".cargo");
    fs::create_dir_all(&cargo_config_dir)?;

    // Create config.toml
    let config_path = cargo_config_dir.join("config.toml");
    let mut config_file = File::create(config_path)?;

    // Fill out cargo.toml
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
    
    // Fill out config.toml and lib.rs
    writeln!(config_file, "[build]")?;
    writeln!(config_file, "target = \"x86_64-pc-windows-msvc\"")?;
    
    writeln!(def_file, "LIBRARY {}", crate_name)?;
    writeln!(def_file, "EXPORTS")?;

    writeln!(lib_file, "use winapi::um::libloaderapi::{{GetProcAddress, LoadLibraryA}};")?;
    writeln!(lib_file, "use std::ffi::CString;")?;
    writeln!(lib_file, "use dllproxymacros::{{prehook, posthook, fullhook}};")?;
    
    for (dll_name, exports) in dlls_and_exports {
        let mut i = 1;

        for export in exports {
            writeln!(def_file, "    {} = {}.{} @{}", export, &dll_name[0..dll_name.len() - 4], export, i)?;

            writeln!(lib_file, "#[no_mangle] //{}", dll_name)?;
            writeln!(lib_file, "fn {}() {{}}", export)?;
            i += 1;
        }
    }

    let full_def_path = fs::canonicalize(&def_path)?.to_string_lossy().to_string();
    let full_def_path = full_def_path.strip_prefix(r"\\?\").unwrap_or(&full_def_path); // Sometimes the extended path marker will not be present, so remove it

    #[cfg(target_os = "windows")]
    {
        writeln!(config_file, "rustflags = [\"-C\", \"link-args=/DEF:{}\"]", full_def_path.replace("\\", "/"))?;
    }
    
    Ok(crate_dir)
}

#[derive(Parser)]
#[command(version = "2.0.0", about = "A simple command-line utility for generating proxy DLLs in Rust", long_about = None)]
struct CLI {
    #[arg(short = 'p', long = "path", value_name = "FILE", help = "Path(s) to original dll(s)")]
    dll_paths: Vec<PathBuf>,

    #[arg(short = 'o', long = "output", help = "Directory to create new crate in, defaults to the running dir")]
    new_dir: Option<String>,

    #[arg(short = 'n', long = "name", help = "Name of the new crate")]
    new_name: String,
}

pub mod parsedllexports;
use std::io::{stdin, stdout};

fn main() {

    print!("Enter DLL Path: ");
    stdout().flush().unwrap();
    let mut dll_path = String::new();
    stdin().read_line(&mut dll_path).unwrap();
    let dll_path = dll_path.trim();

    let path = Path::new(dll_path);
    let dll_name = path.file_name().unwrap().to_string_lossy().to_string();

    print!("Enter new crate directory: ");
    stdout().flush().unwrap();
    let mut new_dir = String::new();
    stdin().read_line(&mut new_dir).unwrap();
    let new_dir = new_dir.trim();

    print!("Enter new crate name: ");
    stdout().flush().unwrap();
    let mut new_name = String::new();
    stdin().read_line(&mut new_name).unwrap();
    let new_name = new_name.trim();

    let exports = parsedllexports::parse_dll_exports(&dll_path).expect("Bad DLL");
    let dependencies = vec!["dllproxymacros = \"0.1.0\"", "winapi = { version = \"0.3.9\", features = [\"libloaderapi\", \"minwindef\"] }"];

    create_rust_lib_crate(new_dir, &new_name, &dll_name, exports, Some(dependencies)).unwrap();

}

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn create_rust_lib_crate<P: AsRef<Path>>(
    dir_path: P, 
    crate_name: &str,
    dll_name: &str, 
    exports: Vec<String>,
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
    
    let mut i = 1;
    for export in exports {
        writeln!(def_file, "    {} = {}_.{} @{}", export, &dll_name[0..dll_name.len() - 4], export, i)?;

        writeln!(lib_file, "#[no_mangle]")?;
        writeln!(lib_file, "fn {}() {{}}", export)?;
        i += 1;
    }

    let full_def_path = fs::canonicalize(&def_path)?.to_string_lossy().to_string();
    let full_def_path = full_def_path.strip_prefix(r"\\?\").unwrap_or(&full_def_path); // Sometimes the extended path marker will not be present, so remove it

    #[cfg(target_os = "windows")]
    {
        writeln!(config_file, "rustflags = [\"-C\", \"link-args=/DEF:{}\"]", full_def_path.replace("\\", "/"))?;
    }
    
    Ok(crate_dir)
}
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

use clap::Args;

#[derive(Args)]
pub struct BuildArgs {
    /// Path to the proxy crate (defaults to current directory).
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Cargo build profile to use.
    #[arg(long, default_value = "release")]
    profile: String,

    #[arg(long)]
    no_build: bool,

    #[arg(last = true, allow_hyphen_values = true)]
    cargo_args: Vec<String>,
}

#[derive(Debug, Clone)]
struct ProxyFunction {
    name: String,
    /// Original DLL name (with the .dll suffix), e.g. "bench64_.dll".
    /// May be `None` if not recoverable from src/lib.rs alone.
    orig_dll: Option<String>,
    hooked: bool,
}

#[derive(Debug, Clone)]
struct DefEntry {
    name: String,
    forward: Option<String>,
    ordinal: u32,
}

pub fn run(args: BuildArgs) -> Result<(), Box<dyn Error>> {
    let crate_dir = fs::canonicalize(&args.path).map_err(|e| {
        format!(
            "could not resolve crate path '{}': {}",
            args.path.display(),
            e
        )
    })?;

    let lib_rs = crate_dir.join("src").join("lib.rs");
    if !lib_rs.is_file() {
        return Err(format!("expected src/lib.rs at {}", lib_rs.display()).into());
    }

    let crate_name = read_crate_name(&crate_dir.join("Cargo.toml"))?;
    let def_path = crate_dir.join(format!("{}.def", crate_name));

    let existing_entries = if def_path.is_file() {
        parse_def(&def_path)?
    } else {
        Vec::new()
    };

    let functions = parse_proxy_lib(&lib_rs)?;

    let new_def = build_def_content(&crate_name, &functions, &existing_entries)?;

    let prev_def = fs::read_to_string(&def_path).ok();
    fs::write(&def_path, &new_def)?;
    match prev_def {
        Some(prev) if prev == new_def => println!("{} already up to date", def_path.display()),
        _ => println!("updated {}", def_path.display()),
    }

    if args.no_build {
        return Ok(());
    }

    // Force recompile
    touch_mtime(&lib_rs)?;

    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--profile")
        .arg(&args.profile)
        .args(&args.cargo_args)
        .current_dir(&crate_dir);

    println!(
        "running: cargo build --profile {} {}",
        args.profile,
        args.cargo_args.join(" ")
    );

    let status = cmd
        .status()
        .map_err(|e| format!("failed to spawn cargo: {}", e))?;
    if !status.success() {
        return Err(format!("cargo build exited with status {}", status).into());
    }

    Ok(())
}

fn build_def_content(
    crate_name: &str,
    functions: &[ProxyFunction],
    existing: &[DefEntry],
) -> Result<String, Box<dyn Error>> {
    let mut by_name: HashMap<&str, &DefEntry> = HashMap::new();
    let mut next_ordinal: u32 = 1;
    for entry in existing {
        by_name.insert(entry.name.as_str(), entry);
        if entry.ordinal >= next_ordinal {
            next_ordinal = entry.ordinal + 1;
        }
    }

    let mut out = String::new();
    out.push_str(&format!("LIBRARY {}\n", crate_name));
    out.push_str("EXPORTS\n");

    for func in functions {
        let ordinal = match by_name.get(func.name.as_str()) {
            Some(e) => e.ordinal,
            None => {
                let n = next_ordinal;
                next_ordinal += 1;
                n
            }
        };

        if func.hooked {
            out.push_str(&format!("    {} @{}\n", func.name, ordinal));
        } else {
            // Determine origdll: prefer source comment / macro arg, then fall back
            // to the existing .def file's forwarding entry.
            let origdll = func
                .orig_dll
                .as_deref()
                .map(strip_dll_ext)
                .map(str::to_owned)
                .or_else(|| {
                    by_name
                        .get(func.name.as_str())
                        .and_then(|e| e.forward.clone())
                });

            let origdll = origdll.ok_or_else(|| {
                format!(
                    "could not determine the original DLL for unhooked function '{}'. \
                     Restore the `//<dllname>.dll` comment after `#[no_mangle]` in src/lib.rs, \
                     or keep the existing forwarding entry in the .def file.",
                    func.name
                )
            })?;

            out.push_str(&format!(
                "    {} = {}.{} @{}\n",
                func.name, origdll, func.name, ordinal
            ));
        }
    }

    Ok(out)
}

fn touch_mtime(path: &Path) -> io::Result<()> {
    let f = fs::OpenOptions::new().write(true).open(path)?;
    f.set_modified(SystemTime::now())?;
    Ok(())
}

fn strip_dll_ext(s: &str) -> &str {
    if s.len() >= 4 && s[s.len() - 4..].eq_ignore_ascii_case(".dll") {
        &s[..s.len() - 4]
    } else {
        s
    }
}

fn read_crate_name(cargo_path: &Path) -> Result<String, Box<dyn Error>> {
    let content = fs::read_to_string(cargo_path)
        .map_err(|e| format!("could not read {}: {}", cargo_path.display(), e))?;

    let mut in_package = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix('[') {
            let header = rest.trim_end_matches(']').trim();
            in_package = header == "package";
            continue;
        }
        if !in_package {
            continue;
        }
        if trimmed.starts_with('#') {
            continue;
        }
        let key = trimmed.split('=').next().map(str::trim).unwrap_or("");
        if key != "name" {
            continue;
        }
        if let Some(eq) = trimmed.find('=') {
            let raw = trimmed[eq + 1..].trim();
            let raw = raw.split('#').next().unwrap_or(raw).trim();
            let value = raw.trim_matches(|c: char| c == '"' || c == '\'');
            if !value.is_empty() {
                return Ok(value.to_string());
            }
        }
    }

    Err(format!(
        "could not find `name` in [package] of {}",
        cargo_path.display()
    )
    .into())
}

fn parse_def(def_path: &Path) -> io::Result<Vec<DefEntry>> {
    let content = fs::read_to_string(def_path)?;
    let mut entries = Vec::new();
    let mut in_exports = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(';') {
            continue;
        }
        let upper = trimmed.to_ascii_uppercase();
        if upper == "EXPORTS" {
            in_exports = true;
            continue;
        }
        if upper.starts_with("LIBRARY") {
            in_exports = false;
            continue;
        }
        if !in_exports {
            continue;
        }

        let (head, ordinal) = match trimmed.split_once('@') {
            Some((h, o)) => (h.trim(), o.trim().parse::<u32>().ok()),
            None => (trimmed, None),
        };
        let ordinal = match ordinal {
            Some(n) => n,
            None => continue,
        };

        let (name, forward) = match head.split_once('=') {
            Some((n, fwd)) => {
                let fwd = fwd.trim();
                let dll = fwd
                    .split('.')
                    .next()
                    .map(str::trim)
                    .unwrap_or("")
                    .to_string();
                (n.trim().to_string(), Some(dll))
            }
            None => (head.trim().to_string(), None),
        };

        if name.is_empty() {
            continue;
        }
        entries.push(DefEntry {
            name,
            forward,
            ordinal,
        });
    }

    Ok(entries)
}

fn parse_proxy_lib(lib_path: &Path) -> Result<Vec<ProxyFunction>, Box<dyn Error>> {
    let content = fs::read_to_string(lib_path)?;
    let mut result = Vec::new();

    let mut pending_kind: Option<&'static str> = None;
    let mut pending_dll: Option<String> = None;

    for line in content.lines() {
        let stripped = line.trim_start();
        if stripped.starts_with("#[") {
            if let Some((kind, dll_from_macro)) = parse_attr_line(stripped) {
                pending_kind = Some(kind);
                pending_dll = dll_from_macro.or_else(|| extract_dll_comment(line));
            }
            continue;
        }

        // Function declaration
        if let Some(name) = extract_fn_name(stripped) {
            if let Some(kind) = pending_kind.take() {
                let dll = pending_dll.take();
                result.push(ProxyFunction {
                    name,
                    orig_dll: dll,
                    hooked: kind != "no_mangle",
                });
            }
        }
    }

    Ok(result)
}

/// Returns (kind, optional dllname-from-macro-args) if the line carries a
/// relevant attribute. Recognizes `#[no_mangle]`, `#[unsafe(no_mangle)]`,
/// and the three hook macros.
fn parse_attr_line(line: &str) -> Option<(&'static str, Option<String>)> {
    let inner = line.trim_start_matches("#[").trim_end();
    // Drop everything from the first trailing comment so we don't match inside it.
    let inner = inner.split("//").next().unwrap_or(inner).trim_end();

    let kind: &'static str =
        if inner.starts_with("no_mangle") || inner.starts_with("unsafe(no_mangle)") {
            "no_mangle"
        } else if inner.starts_with("prehook") {
            "prehook"
        } else if inner.starts_with("posthook") {
            "posthook"
        } else if inner.starts_with("fullhook") {
            "fullhook"
        } else {
            return None;
        };

    let dll = if kind == "no_mangle" {
        None
    } else {
        // Pull first string literal out of the macro args.
        let open = inner.find('(')?;
        let after = &inner[open + 1..];
        let q1 = after.find('"')?;
        let rest = &after[q1 + 1..];
        let q2 = rest.find('"')?;
        Some(rest[..q2].to_string())
    };

    Some((kind, dll))
}

fn extract_dll_comment(line: &str) -> Option<String> {
    let idx = line.find("//")?;
    let comment = line[idx + 2..].trim();
    if comment.is_empty() {
        return None;
    }
    // Take the first whitespace-delimited token; accept anything ending in .dll.
    let token = comment.split_whitespace().next()?;
    if token.to_ascii_lowercase().ends_with(".dll") {
        Some(token.to_string())
    } else {
        None
    }
}

fn extract_fn_name(line: &str) -> Option<String> {
    // Accept lines like:
    //   fn foo(...)
    //   pub fn foo(...)
    //   pub(crate) fn foo(...)
    //   extern "system" fn foo(...)
    //   unsafe fn foo(...)
    let pos = if line.starts_with("fn ") {
        Some(0)
    } else {
        line.find(" fn ").map(|p| p + 1)
    };
    let pos = pos?;
    let rest = &line[pos + 3..];
    let end = rest
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(rest.len());
    if end == 0 {
        None
    } else {
        Some(rest[..end].to_string())
    }
}

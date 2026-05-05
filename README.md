# rustdllproxy

<div align="center">
    <img src="https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white"></img>
    <img src="https://img.shields.io/crates/v/rustdllproxy?style=for-the-badge"></img>
    <img src="https://img.shields.io/crates/d/rustdllproxy?style=for-the-badge"></img>
</div>
<br>
<div align="center">
A Rust crate utility to easily generate and develop proxy DLLs for Windows applications.
</div>

## Installation

```bash
cargo install rustdllproxy
```

## Compatibility

This crate currently only supports the standard DLL PE format. **.NET DLLs are not supported.**

## Use Cases

This crate serves two main purposes:

1. **Single DLL Proxying** - Proxy a single DLL to modify or better understand its behavior
2. **DLL Consolidation** - Collect several DLLs into one unified proxy, which can be used alongside custom applications or techniques

## Current Limitations

- Only supports exports from the standard PE DLL format (**.NET DLLs are not compatible**)
- When hooking functions with custom code, **the function signature must be known**
  - This can be found using disassemblers and reverse engineering tools like [Ghidra](https://ghidra-sre.org/)

## Commands

Rustdllproxy ships two subcommands:

| Command | Purpose |
| --- | --- |
| `rustdllproxy new`   | Generate a new proxy `cdylib` crate from one or more existing DLLs. |
| `rustdllproxy build` | Sync the `.def` file with `src/lib.rs` and build the crate. |

```bash
rustdllproxy --help        # top-level help
rustdllproxy new --help    # generation flags
rustdllproxy build --help  # build flags
```

## Creating a New Crate

```bash
rustdllproxy new -p path/to/target.dll -n my_proxy
```

> **Tip:** Use `-p` multiple times to unify several DLLs into one proxy.

> **Tip:** The `-a` flag can be used to optionally compile for 32 bit.

### Important: Proxy Strategy

Before creating your crate, decide how the proxy DLL will interact with the original(s). A typical pattern is to **append an underscore** to the original DLL name.

**⚠️ THIS MUST BE DONE BEFORE GENERATING THE CRATE** - the generated `.def` file will reference this name for forwarding behavior.

## Writing Hooks

The macro library supports 3 main hook types: `prehook`, `posthook`, and `fullhook`.

### Hook Implementation Steps

1. Replace the `#[no_mangle]` directive with the hook macro (leave the `//<dllname>.dll` trailing comment in place — `rustdllproxy build` uses it to recover the original DLL name):

   ```rust
   #[prehook("dllbeingproxied.dll", "function_name")] //dllbeingproxied.dll
   ```

2. Fill out the function signature (declare inputs as `mut` to modify them)

3. Build with `rustdllproxy build`. The tool will rewrite the `.def` file so that hooked functions no longer forward and unhooked functions still do, then invoke `cargo build --profile release` for you.

> Previously, step 3 required hand-editing the `.def` file (`function_name = dll.function_name @N` → `function_name @N`) and then forcing a Cargo rebuild because Cargo doesn't fingerprint `.def` changes. The `build` subcommand handles both.

### Hook Types

> In this section, target.dll is commonly used. Remember in most cases this would be target\_.dll

#### `prehook`

Executes code **before** the original function. Allows you to add functionality or modify input variables.

```rust
#[prehook("target.dll", "my_function")] //target.dll
fn my_function(mut param1: i32, mut param2: &str) {
    // Your code here - executes before original function
    param1 *= 2;  // Modify parameters if needed
}
```

#### `posthook`

Executes code **after** the original function. View and edit the return value using the magic `ret` variable.

```rust
#[posthook("target.dll", "calculate")] //target.dll
fn calculate(input: i32) -> i32 {
    // Original function executes first
    // Then your code runs with access to 'ret'
    ret = ret * 2;  // Modify return value
}
```

> **Note:** The `ret` variable is automatically defined as mutable. You don't need to reference it if not needed.

#### `fullhook`

Provides **complete control** over function execution. Manually manage the return value and function calling.

```rust
#[fullhook("target.dll", "do_multi_add")] //target.dll
fn do_multi_add(mut a: i32, mut b: i32, mut c: i32) -> i32 {
    // Pre-processing
    a += 10;
    b += 20;

    // Call original function with magic func()
    let mut return_value: i32 = func(a, b, c);

    // Post-processing
    return_value *= 2;

    // Must explicitly return the value
    return_value
}
```

## Building the Crate

Run from the proxy crate directory (or pass it as the first argument):

```bash
rustdllproxy build [PATH] [--profile <name>] [--no-build] [-- <extra cargo args>]
```

| Flag | Default | Effect |
| --- | --- | --- |
| `PATH` | `.` | Path to the proxy crate root. |
| `--profile <name>` | `release` | Cargo build profile (`release`, `dev`, custom). |
| `--no-build` | off | Regenerate the `.def` file but skip `cargo build`. |
| `-- <args>` | — | Forwarded verbatim to `cargo build`. |

### How It Works

1. Reads `[package].name` from `Cargo.toml` to locate `<name>.def`.
2. Walks `src/lib.rs` to classify every exported function as either **hooked** (`#[prehook]` / `#[posthook]` / `#[fullhook]`) or **forwarded** (`#[no_mangle]`).
3. For each function, the original DLL name is recovered from, in order:
   - the first string literal of the hook macro (hooked functions only),
   - the trailing `//<dllname>.dll` comment on the attribute line,
   - the existing forwarding entry already in the `.def` file.
4. Rewrites `<name>.def` so hooked functions read `name @N` and forwarded functions read `name = origdll.name @N`. Existing ordinals are preserved.
5. Bumps `src/lib.rs`'s mtime to force a relink (Cargo doesn't fingerprint `.def` changes), then invokes `cargo build`.

### Caveats

- The `.def` file is **fully regenerated** on every run — manual edits to it (extra directives, custom ordinals) will be overwritten.
- If an `#[no_mangle]` function loses both its `//<dllname>.dll` comment **and** its forwarding entry in the `.def` file, the build aborts with an error explaining how to restore one of them.

## Example Workflow

Let's say you want to modify `office.dll` used in office software via DLL search order hijacking:

### Step 1: Prepare the Original DLL

```bash
# Rename the original DLL
mv office.dll office_.dll
```

### Step 2: Generate Proxy Crate

```bash
rustdllproxy new -p office_.dll -n office_proxy
```

### Step 3: Implement Hooks

Leave the `//office_.dll` trailing comment that the generator emitted — the `build` step reads it.

```rust
#[prehook("office_.dll", "open_window")] //office_.dll
fn open_window() {
    // Your custom code here...
    println!("Window is about to open!");
}
```

### Step 4: Build and Deploy

```bash
cd office_proxy
rustdllproxy build
# Rename the built DLL back to office.dll
# Place in the target directory
```

`rustdllproxy build` reconciles the `.def` file with `src/lib.rs`, then runs `cargo build --profile release`. The crate is force-recompiled on every invocation so the artifact always reflects the current `.def`.

## DLL Bundling Considerations

> **Important:** Proxying several DLLs together is typically useful for **reverse engineering and custom software development**, not process modification.

When bundling multiple DLLs:

- Function **ordinals may change** due to export ordering
- This is rarely problematic since modern software uses export names for compatibility
- Primarily useful for analysis and custom application development

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

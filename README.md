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

This crate currently only supports the standard DLL PE format.

## Current Limitations

- When hooking functions with custom code, **the function signature must be known**
  - This can be found using disassemblers and reverse engineering tools like [Ghidra](https://ghidra-sre.org/)

## Commands

Rustdllproxy ships two subcommands:

| Command              | Purpose                                                             |
| -------------------- | ------------------------------------------------------------------- |
| `rustdllproxy new`   | Generate a new proxy `cdylib` crate from one or more existing DLLs. |
| `rustdllproxy build` | Sync the `.def` file with `src/lib.rs` and build the crate.         |

```bash
rustdllproxy --help        # top-level help
rustdllproxy new --help    # generation flags
rustdllproxy build --help  # build flags
```

## Creating a New Crate

#### A Quick Note on Strategy

Before generating your crate, decide how you would like your proxy to work. A typical pattern is search order hijacking, where you would first rename your target DLL to something like `target_.dll`, and then use the compiled proxy as `target.dll`. This creates a flow resembling `binary -> target.dll -> target_.dll`

There are multiple paths forward depending on your use case. If however you need to rename the underlying DLL being proxied, update the generated `.def` file accordingly.

---
```bash
rustdllproxy new -p path/to/target_.dll -n my_proxy
```

> **Tip:** rustdllproxy is built as a CLI with clap. Run rustdllproxy --help to see all options and flags.

## Writing Hooks

The macro library supports 3 main hook types: `prehook`, `posthook`, and `fullhook`.

### Hook Implementation Steps

1. Replace the `#[no_mangle]` directive with the hook macro (leave the `//<dllname>.dll` trailing comment in place)

   ```rust
   #[prehook("dllbeingproxied.dll", "function_name")] //dllbeingproxied.dll
   ```

2. Fill out the function signature (declare inputs as `mut` to modify them)

3. Build with `rustdllproxy build`.

### Hook Types

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

| Flag               | Default   | Effect                                             |
| ------------------ | --------- | -------------------------------------------------- |
| `PATH`             | `.`       | Path to the proxy crate root.                      |
| `--profile <name>` | `release` | Cargo build profile (`release`, `dev`, custom).    |
| `--no-build`       | off       | Regenerate the `.def` file but skip `cargo build`. |
| `-- <args>`        | —         | Forwarded verbatim to `cargo build`.               |



### Caveats

- The `.def` file is **fully regenerated** on every build, manual changes will be overwritten. If you need to make manual changes against how rustdllproxy builds, cargo can be used to accomplish this.
- The build system relies on generated comments, .def exports, and hook names to retrieve the name of the underlying DLL before building. If there is not enough information, and error will be thrown to explain how this can be recovered.

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
```
> Build files are located under `/target`

## DLL Bundling Considerations

It is possible to proxy several target DLLs with a single crate. This feature is rarely used and comes with some important caveats.

When bundling multiple DLLs:

- Function **ordinals may change** due to export ordering
- This is rarely problematic since modern software uses export names for compatibility
- Primarily useful for analysis and custom application development

## Changelog

Release notes live in [CHANGELOG.md](CHANGELOG.md).

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

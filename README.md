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

## Creating a New Crate

Rustdllproxy generates a `cdylib` crate that compiles into a DLL.

```bash
rustdllproxy --help  # See all available options
```

> **Tip:** Use the `-p` argument multiple times to unify several different DLLs into one proxy.

### Important: Proxy Strategy

Before creating your crate, decide how the proxy DLL will interact with the original(s). A typical pattern is to **append an underscore** to the original DLL name.

**⚠️ THIS MUST BE DONE BEFORE GENERATING THE CRATE** - the generated `.def` file will reference this name for forwarding behavior.

> **Cargo Build Issue:** Cargo will not rebuild if you change a `.def` file. Either force a rebuild or modify `lib.rs` to trigger recompilation.

## Writing Hooks

The macro library supports 3 main hook types: `prehook`, `posthook`, and `fullhook`.

### Hook Implementation Steps

1. Replace the `#[no_mangle]` directive with the hook macro:

   ```rust
   #[prehook("dllbeingproxied.dll", "function_name")]
   ```

2. Fill out the function signature (declare inputs as `mut` to modify them)

3. Update the generated `.def` file by removing forwarding behavior:

   ```diff
   - function_name = dllbeingproxied.function_name @2
   + function_name @2
   ```

4. Build and deploy!

> **Remember:** Force a full Cargo rebuild if you forget to update the `.def` file, as Cargo won't detect the change.

### Hook Types

> In this section, target.dll is commonly used. Remember in most cases this would be target\_.dll

#### `prehook`

Executes code **before** the original function. Allows you to add functionality or modify input variables.

```rust
#[prehook("target.dll", "my_function")]
fn my_function(mut param1: i32, mut param2: &str) {
    // Your code here - executes before original function
    param1 *= 2;  // Modify parameters if needed
}
```

#### `posthook`

Executes code **after** the original function. View and edit the return value using the magic `ret` variable.

```rust
#[posthook("target.dll", "calculate")]
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
#[fullhook("target.dll", "do_multi_add")]
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

## Example Workflow

Let's say you want to modify `office.dll` used in office software via DLL search order hijacking:

### Step 1: Prepare the Original DLL

```bash
# Rename the original DLL
mv office.dll office_.dll
```

### Step 2: Generate Proxy Crate

```bash
rustdllproxy -p office_.dll -n office_proxy
```

### Step 3: Implement Hooks

```rust
#[prehook("office_.dll", "open_window")]
fn open_window() {
    // Your custom code here...
    println!("Window is about to open!");
}
```

### Step 4: Update .def File

```diff
- open_window = office_.open_window @3
+ open_window @3
```

### Step 5: Build and Deploy

```bash
cargo build --release
# Rename the built DLL back to office.dll
# Place in the target directory
```

## DLL Bundling Considerations

> **Important:** Proxying several DLLs together is typically useful for **reverse engineering and custom software development**, not process modification.

When bundling multiple DLLs:

- Function **ordinals may change** due to export ordering
- This is rarely problematic since modern software uses export names for compatibility
- Primarily useful for analysis and custom application development

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

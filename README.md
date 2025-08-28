# rustdllproxy

  

A crate utility to easily generate and develop proxy DLLs.

  

Install with `cargo install rustdllproxy`.

  

There is a video tutorial [here](https://youtu.be/f7WVPpsBXNA).

  

> This video is now outdated. The process of writing functions remains the same, however the crate creation process and naming requirements have changed.

  

# Compatability

  

This crate currently only supports the normal DLL PE format. As such, .NET DLLs are not supported.

  

# Utility

  

This crate serves two main purposes:

  

- Proxying a single DLL in order to modify or better understand its behavior.

  

- Collecting several DLLs into one to modify behavior, which can then be used along side a custom application or technique to use or understand several DLLs as one.

  

**Current Limitations**

  

- Currently, the crate only understands exports from the standard PE DLL format. As such, .NET DLLs are not compatible.

  

- When hooking functions with custom code, the function signature must be known. This can easily be found with a multitude of disassemblers and reverse engineering tools, like Ghidra.

  

# Creating a New Crate

Rustdllproxy generates a `cdylib` crate that can be compiled into a DLL. See `rustdllproxy --help` for more info on using the command.

> Note, the `-p` argument can be used several times to unify several different DLLs into one.

Before creating your crate, consider how you want the proxy DLL to interact with the original(s). A typical pattern is to append and underscore to the name of the original. ***THIS MUST BE DONE BEFORE GENERATING THE CRATE.*** The generated .def file will reference this name for forwarding behavior. This of course can be modified later, but current Cargo behavior makes this a pain in the ass.

> Cargo will not rebuild if you change a .def file. Either force a rebuild or change something within lib.rs to force Cargo to take account of the updated .def

# Writing Hooks

The macro library current supports 3 main hooks: prehook, posthook, and fullhook.

In order to invoke a hook, you must do the following:

1. Replace the `#[no_mangle]` directive with the hook macro, IE `#[prehook("dllbeingproxied.dll", "function_name")]`
2. Fill out the function signature. Remember, you can declare inputs as `mut` to modify them in the function.
3. Go to the generated .def file, and remove the forwarding behavior. `function_name = dllbeingproxied.function_name @2` now becomes `function_name @2`. This tells the compiler to export the new symbol instead of using the default function.
4. All set!

> Remember, if you build and forget to update the .def, force Cargo to do a full rebuild. Cargo will not notice the changed .def file and will serve you a cached build instead.

## prehook

Prehook is the simplest hook. Code you write in a prehook will execute before the normal function. In this time you are able to add functionality or modify input variables.

## posthook

Posthook allows for adding functionality after the original function is executed. This allows you to both view and edit the return value with the magic `ret` variable. If the function returns a value, you can directly write to this variable, IE `ret = 4` to change the return value.

> Note, the macro has already defined ret as mutable. Also, you are not required to reference it if you don't need too.

## fullhook

Full hook allows for full control of what occurs both before and after the function, but in turn adds a small amount of complexity.

When writing a full hook, you must manually manage the return value and the calling of the function via the magic `func()` function, which you must call with the correct arguments in order to signal the execution of the function being proxied.

Additionally, if `func()` has a return value, you must also ensure that the hook stores and returns this value at the end of the hook's execution.

Example:

```rust
#[fullhook("dllbeingproxied.dll", "do_multi_add")]
fn do_multi_add(mut a: i32, mut b: i32, mut c: i32) -> i32 {
	// Do some stuff to a, b, c, or add more code

	let mut return_value: i32 = func(a, b, c);

	// Do some stuff to the return value or add more code

	// Return the value
	return_value
}
```

# A Typical Workflow

Say I want to modify a DLL used in common office software. I plan on using search order hijacking directly in the directory of the DLL, lets call it `office.dll`.

I would elect to rename `office.dll` to `office_.dll`. Then, run `rustdllproxy` with the appropriate arguments.

In the generated crate, I would like the prehook the `open_window` function, which I know following some reverse engineering has no arguments or return type. I would write the following:

```rust
#[prehook("office_.dll", "open_window")]
fn open_window() {
	// Write arbitrary code here...
}
```

Following, edit the .def file from `open_window = office_.open_window @3` to `open_window @3`.

Build as release, and as described earlier, rename the new DLL as if it was the original `office.dll` and move it into the directory.

# Considerations When Bundling

To clear up some confusion, proxying several DLLs together is almost never useful for process modification, it's only useful for reverse engineering and building custom software.

Note that when several DLLs are bundled together, the *ordinals* that functions are exported at may change due to their "spot" being taken by another DLL's functions.
This should never really be a problem, as most modern software uses export names anyways for compatability and readability in their code.


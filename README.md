# rustdllproxy

A crate utility to easily generate and develop proxy DLLs.

Install with `cargo install rustdllproxy`.

There is a video tutorial [here](https://youtu.be/f7WVPpsBXNA).


# Creating a New Library

To create a new library, navigate to a good directory close to the target DLL, and then run the `rustdllproxy` command in your terminal. Follow the prompts for the DLL path, new crate directory, and the new name of your crate.

> IMPORTANT! Select the location of your crate wisely. It cannot be easily moved after creation. If you do, please update the path of the .def file in the linker options.

Now the library crate should have been created.

# Hooking Into Functions

Now that you are in the new library, you should see some boilerplate generated in `lib.rs`. This is required for forwarding exports from the old DLL, so keep it untouched unless it is hooked.

In order to hook a function, you must replace the `no_mangle` macro with any of the following:

- prehook
- posthook
- fullhook

in the format `#[prehook("dll_being_hooked", "function_name")]`.

> Please also note, that the name of the function should be untouched. If it is changed, the linker will have problems when generating exports.

Once you have used the macro, you must fill out the signature of the function. This is a current limitation that will be fixed soon, but for hooking currently you must know the function's signature. This can be determined with a multitude of different tools and techniques. If you are using this tool, I assume that you are already using some sort of disassembler or decompiler to look at said DLL.

Finally, you must visit the .def file and remove the forwarding behavior such that just the function name being exported remains (ex.)

`do_multi_add = yourdll_.do_multi_add @1` turns into `do_multi_add @1`, assuming you are hooking do_multi_add.

Finally, run `cargo build --release` to build your DLL.

> It is important that you change the .def file before compilation. If you compile before changing the .def file, it will obviously not be hooked. If you change the .def file and try again, it will still not be hooked. Rather, cargo will return the cached build. This behavior is due to cargo not searching for differences in the .def file. Save yourself the headache and do it right the first time.

## prehook

When prehooking you can modify the values of the input arguments. When you create the function signature, you may define arguments as mutable as you normally would in Rust, and changing these arguments works as expected.

## posthook

When writing a posthook, if your function is returning something, you can modify the value with the magic `ret` variable. Changing the value of ret will change what the function returns.

## fullhook

A fullhook is more advanced but gives you a deeper level of control over the hook. A fullhook relies on you to call the original function manually using the magic `func` function. This function must have the original arguments passed to it as well. If your function has a return value, it must also be stored and then returned manually at the end of your hook.

# Using the DLL

The new DLL generated must be placed in the same directory as the target DLL. Along with that, the target DLL's name must be appended with an _. For example, `dlltest.dll` becomes `dlltest_.dll`, and you name the proxy DLL to `dlltest.dll`. This is to take advantage of the DLL search order.

There are obviously cases where the approach is slightly different, I assume you know what you are doing for your specific case.
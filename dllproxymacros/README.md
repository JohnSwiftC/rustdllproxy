# dllproxymacros

A series of macros to be used in a seperate project to easily generate Rust proxy DLLs in more modern versions of the language.

# Quick Guide

Not intended to be used on their own but they can, decorate the name of the function that must be exported, the same as the DLLs exported function name with the following:

`#[hooktype("dllnameliteral", "functionnameliteral")]`

Where hooktype is either prehook, posthook, or fullhook.

Whne post hooking, the return value can be modified with the magic `ret` variable.

In a full hook, arguments must be passed and the original function call must be made with the magic `func` function.
If the function returns, it must also return.

For all cases, the function you write must have the same signature as that of the DLL function you are trying to proxy. A prehook without this limitation is coming soon for the main crate.
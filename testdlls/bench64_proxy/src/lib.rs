use dllproxymacros::{fullhook, posthook, prehook};
use std::ffi::CString;
use winapi::um::libloaderapi::{GetProcAddress, LoadLibraryA};

#[prehook("bench64_.dll", "add")] //bench64_.dll
fn add(a: u64, b: u64) -> u64 {
    println!("Add prehook executing!");
}
#[no_mangle] //bench64_.dll
fn add_then_mult() {}
#[fullhook("bench64_.dll", "mult")] //bench64_.dll
fn mult(a: u64, b: u64) -> u64 {
    println!("Hello mult!");

    let mut return_value = func(a, b);

    return_value = 0;

    return_value
}

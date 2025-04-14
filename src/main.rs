pub mod parsedllexports;

fn main() {
    let exports = parsedllexports::parse_dll_exports("../rustdll/shitty.dll").unwrap();
    println!("{:#?}", exports);
}

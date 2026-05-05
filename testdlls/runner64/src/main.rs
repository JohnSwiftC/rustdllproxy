#[link(name = "bench64.dll")]
unsafe extern "C" {
    fn add(a: u64, b: u64) -> u64;
    fn mult(a: u64, b: u64) -> u64;
    fn add_then_mult(a: u64, b: u64, c: u64) -> u64;
}

fn main() {
    println!("Before add call, a = 1, b = 6\n");
    let res = unsafe { add(1, 6) };
    println!("After add call, val = {}\n", res);

    println!("Before mult call, a = 5, b = 5\n");
    let res = unsafe { mult(5, 5) };
    println!("After mult call, val = {}\n", res);

    println!("Before add_then_mult call, a = 5, b = 5, c = 2\n");
    let res = unsafe { add_then_mult(5, 5, 2) };
    println!("After add_then_mult call, val={}\n", res);
}

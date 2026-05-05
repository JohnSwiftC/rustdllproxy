pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub fn mult(left: u64, right: u64) -> u64 {
    left * right
}

pub fn add_then_mult(left: u64, right: u64, factor: u64) -> u64 {
    mult(add(left, right), factor)
}

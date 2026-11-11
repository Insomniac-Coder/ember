#$ test: compile-pass
#$ rules: BLD-11, STD-6
## `[BLD-11]`, `[STD-6]` — a package that declares `layers = ["core"]` may use core: options,
## results, integers and fixed arrays.

pub fn checksum(data: ref [u8; 4]) -> Option[u32]:
    total: u32 = 0
    for byte in data:
        total = total * 31 + byte as u32
    if total == 0:
        return None
    return Some(max(total, 7))

pub fn clamp_to(x: i32, hi: i32) -> Result[i32, i32]:
    if x > hi:
        return Err(x - hi)
    return Ok(x)

fn main():
    data: [u8; 4] = [1, 2, 3, 4]
    sum = checksum(data)
    clamped = clamp_to(9, 5)
    if sum.is_none() or clamped.is_ok():
        return

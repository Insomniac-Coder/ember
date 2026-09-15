#$ test: run-pass
#$ rules: RNG-4, RNG-10
#$ profiles: debug, release, shipping
#$ stdout: 28

type Byte = i32 in 0 ..= 255
type Masked = i32 in 0 ..= 15
type Flags = i32 in 0 ..= 15

fn masked(value: Byte) -> Masked:
    return value & 15

fn or_constants() -> Flags:
    return 4 | 8

fn xor_constants() -> Flags:
    return 4 ^ 1

fn and_constants() -> Flags:
    return 3 & 1

fn main():
    a: i32 = masked(Byte.clamped(42))
    b: i32 = or_constants()
    c: i32 = xor_constants()
    d: i32 = and_constants()
    println(a + b + c + d)

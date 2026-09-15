#$ test: run-pass
#$ rules: RNG-4, RNG-10
#$ profiles: debug, release, shipping
#$ stdout: 30

type Byte = i32 in 0 ..= 255
type NarrowMask = i32 in 0 ..= 15
type Masked = i32 in 0 ..= 15
type Flags = i32 in 0 ..= 15

fn masked(value: Byte) -> Masked:
    return value & 15

fn pairwise(value: Byte, mask: NarrowMask) -> Masked:
    v: i32 = value
    m: i32 = mask
    return v & m

fn or_constants() -> Flags:
    return 4 | 8

fn xor_constants() -> Flags:
    return 4 ^ 1

fn and_constants() -> Flags:
    return 3 & 1

fn main():
    a: i32 = masked(Byte.clamped(42))
    e: i32 = pairwise(Byte.clamped(42), NarrowMask.clamped(7))
    b: i32 = or_constants()
    c: i32 = xor_constants()
    d: i32 = and_constants()
    println(a + b + c + d + e)

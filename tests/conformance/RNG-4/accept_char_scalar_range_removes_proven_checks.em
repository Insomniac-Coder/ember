#$ test: run-pass
#$ rules: RNG-4, TYP-3, TYP-6, TYP-8, FFI-8
#$ profiles: debug, release, shipping
#$ stdout: 1 66 1114112 4294967295 1
#$ assert-c-count: contains("ember_ck_add_u32(") == 2
# Every valid char is at most 0x10FFFF, so widening it to u32 then
# adding one cannot overflow. An addition that may overflow stays checked.
# Foreign char32_t is u32, whose values are not restricted to Unicode scalars:
# an unrestricted u32 parameter must keep its overflow check too.
# @noinline ensures literal call arguments cannot prove these checks safe.

@noinline
fn bump(c: char) -> u32:
    return (c as u32) + 1u32

@noinline
fn still_checked(c: char) -> u32:
    return (c as u32) + 4294967295u32

@noinline
fn unrestricted_bump(value: u32) -> u32:
    return value + 1u32

fn main():
    match char.from_u32(0x10FFFF):
        Some(maximum):
            println(bump('\0'), bump('A'), bump(maximum), still_checked('\0'), unrestricted_bump(0))
        None:
            panic("maximum Unicode scalar unexpectedly rejected")

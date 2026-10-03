#$ test: run-pass
#$ rules: SIMD-5, SIMD-7, OPT-2, TYP-6, TYP-8, CG-C-1
#$ profiles: debug, release, shipping
#$ stdout: 510 -2 510
#$ assert-c: !contains("ember_sum_bytes(")
# Changing sign at byte width or narrowing a widened byte changes its numeric
# value before summing. Reading each original byte as its original sign fails.

@noinline
@overflow(wrap)
fn signed_to_unsigned(xs: Span[i8]) -> int:
    total = 0
    for x in xs:
        converted: u8 = x as u8
        total += converted as int
    return total

@noinline
@overflow(wrap)
fn unsigned_to_signed(xs: Span[u8]) -> int:
    total = 0
    for x in xs:
        converted: i8 = x as i8
        total += converted as int
    return total

@noinline
@overflow(wrap)
fn widened_then_narrowed(xs: Span[i8]) -> int:
    total = 0
    for x in xs:
        widened: i64 = x as i64
        narrowed: u8 = widened as u8
        total += narrowed as int
    return total

fn main():
    signed: Array[i8] = [-128, -1, 0, 127]
    unsigned: Array[u8] = [0, 127, 128, 255]
    println(signed_to_unsigned(signed[..]), unsigned_to_signed(unsigned[..]), widened_then_narrowed(signed[..]))

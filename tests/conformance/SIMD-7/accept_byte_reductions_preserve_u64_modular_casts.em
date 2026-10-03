#$ test: run-pass
#$ rules: SIMD-5, SIMD-7, OPT-2, TYP-6, TYP-8, CG-C-1
#$ profiles: debug, release, shipping
#$ stdout: unsigned byte sums ok
#$ assert-c-count: contains("ember_sum_bytes(") == 4
#$ assert-c: contains(", true)")
#$ assert-c: contains(", false)")
# Each answer is an independently calculated sum modulo 2^64. Negative i8
# values pass through i64 and u64 casts before adding or subtracting. Both
# unsigned operations cross their wrap boundary; all 256 byte values occur.

@noinline
@overflow(wrap)
fn add_unsigned(xs: Span[u8], start: u64) -> u64:
    total = start
    for x in xs:
        total += x as u64
    return total

@noinline
@overflow(wrap)
fn subtract_unsigned(xs: Span[u8], start: u64) -> u64:
    total = start
    for x in xs:
        total -= x as u64
    return total

@noinline
@overflow(wrap)
fn add_signed(xs: Span[i8], start: u64) -> u64:
    total = start
    for x in xs:
        widened: i64 = x as i64
        converted: u64 = widened as u64
        total += converted
    return total

@noinline
@overflow(wrap)
fn subtract_signed(xs: Span[i8], start: u64) -> u64:
    total = start
    for x in xs:
        total -= x as u64
    return total

fn main():
    xs: Array[u8] = []
    ys: Array[i8] = []
    for i in 0..256:
        xs.push(i as u8)
        ys.push((i - 128) as i8)
    assert(add_unsigned(xs[..], 18446744073709551600) == 32624 as u64)
    assert(subtract_unsigned(xs[..], 17) == 18446744073709518993u64)
    assert(add_signed(ys[..], 5) == 18446744073709551493u64)
    assert(subtract_signed(ys[..], 18446744073709551600) == 112 as u64)
    assert(add_signed(ys[1..], 5) == 5 as u64)
    assert(subtract_unsigned(xs[1..1], 17) == 17 as u64)
    println("unsigned byte sums ok")

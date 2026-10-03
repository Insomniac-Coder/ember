#$ test: run-pass
#$ rules: SIMD-5, SIMD-7, OPT-2, TYP-8, FN-9, CG-C-1
#$ profiles: debug, release, shipping
#$ stdout: 9 5
#$ assert-c: !contains("ember_sum_bytes(")
# These addresses are taken before the reduction, by calls that must remain
# in MIR. The byte selection conservatively refuses either address escape,
# independently of whether this particular callee retains the address.

@noinline
@overflow(wrap)
fn touch_total(mut value: u64):
    value += 1

@noinline
@overflow(wrap)
fn touch_counter(mut value: usize):
    value += 1

@noinline
@overflow(wrap)
fn addressed_total(xs: Span[u8]) -> u64:
    total: u64 = 2
    touch_total(total)
    for x in xs:
        total += x as u64
    return total

@noinline
@overflow(wrap)
fn addressed_counter(xs: Span[u8]) -> u64:
    total: u64 = 0
    index: usize = 0
    touch_counter(index)
    limit = xs.len() as usize
    while index < limit:
        total += xs[index] as u64
        index += 1
    return total

fn main():
    xs: Array[u8] = [1, 2, 3]
    println(addressed_total(xs[..]), addressed_counter(xs[..]))

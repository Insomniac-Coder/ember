#$ test: run-pass
#$ rules: SIMD-5, SIMD-7, OPT-2, TYP-8, BRW-2, CG-C-1
#$ profiles: debug, release, shipping
#$ stdout: copied byte sum ok
#$ assert-c: contains("ember_sum_bytes(")
# An explicit index copy, element reference, reference copy and widening all
# describe the same stream. The nonzero range starts unaligned and has a tail.

@noinline
@overflow(wrap)
fn sum(xs: Span[u8], first: usize, last: usize) -> u64:
    total: u64 = 3
    for i in first..last:
        index: usize = i
        item: ref u8 = ref xs[index]
        alias: ref u8 = item
        value: u64 = alias as u64
        total += value
    return total

fn main():
    xs: Array[u8] = []
    for i in 0..128:
        xs.push(i as u8)
    assert(sum(xs[..], 1, 127) == 8004 as u64)
    assert(sum(xs[..], 17, 17) == 3 as u64)
    assert(sum(xs[..], 17, 4) == 3 as u64)
    println("copied byte sum ok")

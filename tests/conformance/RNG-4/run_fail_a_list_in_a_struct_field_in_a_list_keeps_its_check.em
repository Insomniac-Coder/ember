#$ test: run-fail
#$ rules: RNG-4
#$ profiles: debug, development, release
#$ panics: integer overflow
# `[RNG-4]` (ADR-142) — a struct's field reached through a list of structs is
# the same field: `grow`'s push counts.

struct Bin:
    items: Array[i32]

fn grow(ref mut bins: Array[Bin], v: i32):
    bins[0].items.push(owns v)

fn main():
    bins: Array[Bin] = []
    xs: Array[i32] = []
    bins.push(owns Bin(xs))
    for i in 0..10:
        bins[0].items.push(owns ((i % 7) as i32))
    grow(ref mut bins, 2147483642)
    grow(ref mut bins, 0)
    total: i32 = 0
    for i in 0..12:
        total = total + bins[0].items[i]
    println(total)

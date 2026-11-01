#$ test: compile-fail
#$ rules: LT-44, LT-1a
# `[LT-44]` — `from arena` is needed when the function has other source parameters too: `from xs`
# says the result points into `xs` only, so an allocation from the arena is `E3062` (`[LT-1a]`).

fn allocate_one(ref mut arena: FixedArena, xs: Span[int]) -> ref mut i32 from xs:
    return arena.alloc(1)    #$ error[E3062]: `allocate_one` returns a view of `arena`, but its first line says `from xs`

fn main():
    bytes: [u8; 64] = [0; 64]
    fixed: FixedArena = Arena.fixed(bytes)
    ys = [1]
    println(allocate_one(ref mut fixed, ys))

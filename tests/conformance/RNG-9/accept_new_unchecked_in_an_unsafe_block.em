#$ test: run-pass
#$ rules: RNG-9
#$ profiles: debug, development, release
#$ stdout: 50
## `[RNG-9]` — `T.new_unchecked(v)` inside an `unsafe` block, for a `v` in the range.

type Percent = u8 in 0 ..= 100

fn main():
    unsafe:  # SAFETY: 50 is in 0 ..= 100
        ok = Percent.new_unchecked(50)
        shown: u32 = ok
        println(shown)

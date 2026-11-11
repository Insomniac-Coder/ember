#$ test: run-fail
#$ rules: RNG-9
#$ profiles: debug
#$ panics: `Percent.new_unchecked`: the value is not in the range of `Percent`
## `[RNG-9]` — `T.new_unchecked(v)` asserts the range in `debug`: a `v` outside it panics.

type Percent = u8 in 0 ..= 100

fn main():
    v: u8 = 200
    unsafe:  # SAFETY: deliberately wrong, to see the debug assertion
        bad = Percent.new_unchecked(v)
        shown: u32 = bad
        println(shown)

#$ test: compile-fail
#$ rules: RNG-9
## `[RNG-9]` — producing a range value outside its range is undefined behaviour, so a route that
## does not check is `unsafe`: `T.new_unchecked(v)` outside an `unsafe` block is `E3100`.

type Percent = u8 in 0 ..= 100

fn make(v: u8) -> Percent:
    return Percent.new_unchecked(v)    #$ error[E3100]: `Percent.new_unchecked` requires `unsafe`

fn main():
    shown: u32 = make(5)
    println(shown)

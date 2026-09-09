#$ test: compile-fail
#$ rules: FFI-26, RNG-10b
# A range type is representable — it erases to its representation — and is
# still refused at a foreign boundary. `[RNG-10b]`: a value arriving from
# foreign code "enters at the representation type and becomes a range value
# only through `[RNG-3]`". Admitting one here would let a foreign caller
# manufacture a range value that passed no check, and `[RNG-9]` makes that
# undefined behaviour rather than merely a wrong number.

type Roughness = f32 in 0.0 ..= 1.0

extern "C" fn shade(r: Roughness) -> i32:   #$ error[E5054]: `Roughness` is a range type, so it may not cross a foreign boundary
    return 1

fn main():
    println(1)

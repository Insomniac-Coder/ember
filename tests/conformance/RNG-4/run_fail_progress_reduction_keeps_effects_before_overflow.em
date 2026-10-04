#$ test: run-fail
#$ rules: RNG-4, TXT-10, TYP-8
#$ profiles: debug, release, shipping
#$ stdout: seen 0
#$ panics: integer overflow in `+`

@noinline
fn effectful(text: str, initial: int) -> int:
    total = initial
    for (i, c) in text.char_indices():
        println("seen", i)
        total += i ^ (c as u32 as int)
    return total

fn main():
    println(effectful("ab", 9223372036854775807))

#$ test: run-fail
#$ rules: RNG-4, TXT-10, TYP-8
#$ profiles: debug, release, shipping
#$ stdout: 9223372036854775807
#$ panics: integer overflow in `+`

@noinline
fn reduce(text: str, initial: int) -> int:
    total = initial
    for (i, c) in text.char_indices():
        total += i ^ (c as u32 as int)
    return total

fn main():
    println(reduce("", 9223372036854775807))
    println(reduce("a", 9223372036854775807))

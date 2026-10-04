#$ test: run-pass
#$ rules: RNG-4, TXT-10, TYP-8
#$ profiles: debug, release, shipping
#$ stdout: 129027
#$ stdout: -9223372036854646781
#$ stdout: 0
#$ stdout: 9223372036854775807
#$ stdout: -9223372036854775808
#$ stdout: 129020

@noinline
fn reduce(text: str, initial: int) -> int:
    total = initial
    for (i, c) in text.char_indices():
        total += i ^ (c as u32 as int)
    return total

@noinline
fn reduce_skipping_nul(text: str) -> int:
    total = 0
    for (i, c) in text.char_indices():
        if c == '\0':
            continue
        total += i ^ (c as u32 as int)
    return total

fn main():
    text = "aé🙂\0z"
    println(reduce(text, 0))
    println(reduce(text, -9223372036854775807 - 1))
    println(reduce("", 0))
    println(reduce("", 9223372036854775807))
    println(reduce("", -9223372036854775807 - 1))
    println(reduce_skipping_nul(text))

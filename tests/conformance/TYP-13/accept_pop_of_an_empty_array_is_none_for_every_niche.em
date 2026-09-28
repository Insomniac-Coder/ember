#$ test: run-pass
#$ rules: TYP-13, STD-15, CTL-1
#$ profiles: debug, release, shipping
#$ stdout: None None None None
#$ stdout: 2 1 1
# `[TYP-13]` — an `Option` with a niche stores `None` as the niche value (`2` for a
# `bool`, `0x110000` for a `char`, a null view, an unused discriminant), not as zero
# bytes. `pop` of an empty `Array` returned zero bytes, which is `Some(false)`,
# `Some('\x00')` or `Some('')` for these payloads, and an owned `for` loop, which
# pops, never ended (B1).

enum Light:
    Red
    Green

fn main():
    flags: Array[bool] = []
    letters: Array[char] = []
    words: Array[str] = []
    lights: Array[Light] = []
    println(flags.pop(), letters.pop(), words.pop(), lights.pop())
    n = 0
    for b in owned [true, false]:
        if b or not b:
            n += 1
    m = 0
    for w in owned ["only"]:
        m += w.len() - 3
    k = 0
    for c in owned ['x']:
        if c == 'x':
            k += 1
    println(n, m, k)

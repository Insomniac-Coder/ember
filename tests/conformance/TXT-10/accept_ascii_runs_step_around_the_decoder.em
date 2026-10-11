#$ test: run-pass
#$ rules: TXT-10, TXT-2
#$ profiles: debug, release, shipping
#$ assert-c: contains("< 128U")
#$ stdout: 0 0 -1
#$ stdout: 97 1 -1
#$ stdout: 0 1 -1
#$ stdout: 4066 2 1
#$ stdout: 4093 2 0
#$ stdout: 2988107 4 3
#$ stdout: 7039552 4 0
#$ stdout: 100573832 5 2
#$ stdout: 128578 1 0
#$ stdout: 7676220 4 2
#$ stdout: 993330142 58 47
#$ stdout: 354720838 7 0
#$ stdout: 89832259 5 4
#$ stdout: abé🙂c
#$ stdout: 1113

## D-498 (ADR-155) — a loop over a text's characters takes an ASCII byte
## without the UTF-8 decoder, in a copy of the loop that comes back to the
## byte test. Every boundary between the two paths gives the same characters
## and indices as the decoder alone: an empty text, one byte, NUL, DEL (the
## last ASCII byte) next to U+0080 (the first two-byte character), ASCII runs
## before, between and after characters of two, three and four bytes, and a
## loop body that leaves early or skips.

@noinline
fn sum(text: str) -> int:
    total = 0
    for (i, c) in text.char_indices():
        total = total * 31 + (i ^ (c as u32 as int))
        total = total % 1000000007
    return total

@noinline
fn count(text: str) -> int:
    n = 0
    for _c in text:
        n += 1
    return n

@noinline
fn first_wide(text: str) -> int:
    for (i, c) in text.char_indices():
        if c as u32 >= 128:
            return i
    return -1

@noinline
fn skip_spaces(text: str) -> String:
    out = String.from("")
    for c in text.chars():
        if c == ' ':
            continue
        out.push(c)
    return out

@noinline
fn pairs(a: str, b: str) -> int:
    total = 0
    for (i, x) in a.char_indices():
        for (j, y) in b.char_indices():
            if x == y:
                total += i * 100 + j
    return total

fn main():
    texts = ["", "a", "\0", "\x7f\u{80}", "\u{80}\x7f", "abcé", "éabc", "ab€cd", "🙂", "xy🙂z",
             "the quick brown fox, jumps over\tthe lazy dog; héllo wörld\n", "€€€a€€€", "a\0b\0🙂"]
    for t in texts:
        println(sum(t), count(t), first_wide(t))
    println(skip_spaces("a b  é 🙂 c "))
    println(pairs("aé🙂b", "b🙂éa"))

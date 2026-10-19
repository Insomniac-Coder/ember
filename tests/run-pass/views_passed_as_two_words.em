#$ test: run-pass
#$ profiles: debug, development, release
#$ stdout: 6 3 [2, 4, 6] 21 5
# D-569: on Windows a function only ever called directly takes a `str` or a
# span parameter as its pointer and length; one used as a value keeps the
# view whole. Either way the program is the same.

@noinline
fn count_spaces(text: str, start: int) -> int:
    n = start
    for c in text.chars():
        if c == ' ':
            n += 1
    return n

@noinline
fn double_all(mut xs: MutSpan[int], s: Span[int]):
    for i in 0..xs.len():
        xs[i] = s[i] * 2

@noinline
fn digits(text: str) -> int:
    if text.len() == 0:
        return 0
    return text.len() + digits(text[1..])

fn first_len(text: str) -> int:
    return text.len()

fn apply(f: fn(str) -> int, text: str) -> int:
    return f(text)

fn main():
    words = "a b c d e f g"
    a: Array[int] = [0, 0, 0]
    b: Array[int] = [1, 2, 3]
    double_all(a, b)
    println(count_spaces(words, 0), count_spaces("x y z w", 0), a, digits("abcdef"), apply(first_len, "hello"))

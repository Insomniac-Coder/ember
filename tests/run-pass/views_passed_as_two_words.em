#$ test: run-pass
#$ profiles: debug, development, release
#$ stdout: 6 3 [2, 4, 6] 21 5 2
# D-569: on Windows a function only ever called directly takes a `str` or a
# span parameter as its pointer and length; one used as a value keeps the
# view whole. Either way the program is the same. (ODR-278: a function given
# by name, `word_len`, is its own type and is called directly; `held`, a
# `fn(str) -> int`, holds `first_len`'s address.)

@noinline
fn count_spaces(text: StrView, start: int) -> int:
    n = start
    for c in text.chars():
        if c == ' ':
            n += 1
    return n

@noinline
fn double_all(ref mut xs: MutSpan[int], s: Span[int]):
    for i in 0..xs.len():
        xs[i] = s[i] * 2

@noinline
fn digits(text: StrView) -> int:
    if text.len() == 0:
        return 0
    return text.len() + digits(text[1..])

fn first_len(text: StrView) -> int:
    return text.len()

@noinline
fn word_len(text: StrView) -> int:
    return text.len()

fn apply(f: fn(StrView) -> int, text: StrView) -> int:
    return f(text)

fn main():
    words = "a b c d e f g"
    a: Array[int] = [0, 0, 0]
    b: Array[int] = [1, 2, 3]
    double_all(ref mut a, b)
    held: fn(StrView) -> int = first_len
    println(count_spaces(words, 0), count_spaces("x y z w", 0), a, digits("abcdef"), apply(held, "hello"), apply(word_len, "hi"))

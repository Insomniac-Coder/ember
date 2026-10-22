#$ test: run-pass
#$ rules: TXT-10
#$ profiles: debug, development, release
#$ stdout:
#$ true true false false
#$ true true true false
#$ true true true false
#$ true true true false
#$ true false true false
#$ 3 2 1
# `[TXT-10]` (ODR-183) — a `char`'s classes as Python's `isalpha`, `isdigit`, `isalnum` and
# `isspace` of one character give them (Unicode), and the same restricted to ASCII.

fn main():
    println('a'.is_alphabetic(), 'é'.is_alphabetic(), '1'.is_alphabetic(), '_'.is_alphabetic())
    println('7'.is_digit(), '٣'.is_digit(), '²'.is_digit(), 'x'.is_digit())
    println('½'.is_alphanumeric(), 'Z'.is_alphanumeric(), '9'.is_alphanumeric(), '-'.is_alphanumeric())
    println(' '.is_whitespace(), '\u{3000}'.is_whitespace(), '\u{1c}'.is_whitespace(), 'a'.is_whitespace())
    println('a'.is_ascii_alphabetic(), 'é'.is_ascii_alphabetic(), '7'.is_ascii_digit(), '٣'.is_ascii_digit())
    letters = 0
    digits = 0
    spaces = 0
    for c in "ab9 7c":
        if c.is_alphabetic():
            letters += 1
        elif c.is_digit():
            digits += 1
        elif c.is_whitespace():
            spaces += 1
    println(letters, digits, spaces)

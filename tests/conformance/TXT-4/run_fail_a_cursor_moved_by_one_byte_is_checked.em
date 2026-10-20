#$ test: run-fail
#$ rules: TXT-4
#$ profiles: debug, development, release
#$ panics: a slice bound is not on a character boundary
## `[TXT-4]` (D-759) — only a bound known to be a character boundary goes unchecked: an index one byte
## past a character's start is checked, and inside `é` panics.

fn main():
    s = "héllo"
    for (i, c) in s.char_indices():
        if c == 'é':
            println(s[0..i + 1])

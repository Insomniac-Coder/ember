#$ test: run-fail
#$ rules: TXT-4
#$ profiles: debug, development, release
#$ panics: a slice bound is not on a character boundary
## `[TXT-4]` - a `String` is sliced by the same rule: an end bound inside a character panics.

fn main():
    s = String("héllo")
    println(s[0..2])

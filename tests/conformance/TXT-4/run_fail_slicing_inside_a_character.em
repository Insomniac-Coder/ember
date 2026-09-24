#$ test: run-fail
#$ rules: TXT-4
#$ profiles: debug, release, shipping
#$ panics: a slice bound is not on a character boundary
# `[TXT-4]` — a bound inside a character panics in every profile.

fn main():
    s = "héllo"
    println(s[0..2])

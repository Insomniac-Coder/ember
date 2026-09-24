#$ test: run-pass
#$ rules: TXT-4
#$ profiles: debug, release, shipping
#$ stdout: h llo 2
#$ cd
# `[TXT-4]` — `s[a..b]` slices text by byte offset: `é` is two bytes.

fn main():
    s = "héllo"
    println(s[0..1], s[3..], s[1..3].len())
    word: String = "abcdef"
    println(word[2..4])

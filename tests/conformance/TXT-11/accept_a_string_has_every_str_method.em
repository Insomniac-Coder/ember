#$ test: run-pass
#$ rules: TXT-11, TXT-10
#$ profiles: debug, release, shipping
#$ stdout: 3 2 2 Some(2)
#$ ab
#$ ababab 6
# `[TXT-11]` (D-472) — a `String` has every method a `str` has, std's too:
# its iterators and searches run over a view of its text. A counted loop
# pushing a literal on every turn has its room reserved first (ADR-106).

fn main():
    text = String.from("a\nb c")
    words = 0
    for _ in text.split_whitespace():
        words += 1
    println(words, text.lines().count(), text.split(" ").count(), text.find("b"))
    for c in String.from("a b").chars():
        if c != ' ':
            print(c)
    println()
    built = String.from("")
    for _ in 0..3:
        built.push_str("ab")
    println(built, built.len())

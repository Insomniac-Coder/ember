#$ test: compile-fail
#$ rules: STD-19
#$ profiles: debug
#$ error[E2040]: `copied` needs `String` to be `Copy`
# `[STD-19]` — `copied()` copies the values its references reach, so they
# must be `Copy`; a `String` is cloned instead (`cloned()`).

fn main():
    words: Array[String] = [String.from("a")]
    println(words.iter().copied().count())

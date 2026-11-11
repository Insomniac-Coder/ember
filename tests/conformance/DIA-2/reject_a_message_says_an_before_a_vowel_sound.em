#$ test: compile-fail
#$ rules: DIA-2
# `[DIA-2]` — a message reads as a sentence: the article before a quoted name is chosen from the
# word that follows it (`an `Array[i64]``, `an `owns Player``, `a `String``), whatever message
# it is in.

class Player:
    name: String

const LIST: Array[int] = [1, 2]     #$ error[E2130]: a `const` cannot hold an `Array[i64]`

fn main():
    println(LIST.len())

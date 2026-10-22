#$ test: compile-fail
#$ rules: CTL-5, ENM-2
## `[CTL-5]` — the arms MUST be exhaustive. Text has too many values to list, so
## string literal patterns cover only what they name: a `match` on text needs
## `_` or a binding (D-815). A string literal does not match a `char` or an
## integer.

fn code(word: str) -> int:
    return match word:    #$ error[E2090]: `_` not covered
        "north" => 1
        "south" => 2

fn letter(c: char) -> int:
    match c:
        "a":    #$ error[E2020]: this literal cannot match a value of type `char`
            return 1
        _:
            return 0

fn main():
    println(code("north"), letter('a'))

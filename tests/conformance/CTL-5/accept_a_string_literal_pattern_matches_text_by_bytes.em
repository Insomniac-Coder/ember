#$ test: run-pass
#$ rules: CTL-5, TYP-37
#$ profiles: debug, development, release
#$ stdout: 1 1 2 2 0 0 0
#$ 3 0 4
#$ gram litre unknown 'cup'
#$ quit
#$ assert-c: contains("ember_str_eq(")
#$ assert-c: !contains("ember_str_cmp")
## III.6 — `literal_pattern := … | STRING`: a string literal matches text by its
## bytes, the lengths first, with no allocation; `|` alternatives, a binding
## after the literals, a `const` of text, a `String` scrutinee and an empty
## literal all work. A longer or shorter text with the same start, one with a
## NUL inside and one that differs only by case match nothing (D-815).

const QUIT: str = "quit"

fn direction(word: str) -> int:
    return match word:
        "north" | "n" => 1
        "south" | "s" => 2
        _ => 0

fn special(word: str) -> int:
    match word:
        "":
            return 3
        "a\0b":
            return 4
        _:
            return 0

fn unit(word: str) -> String:
    return match word:
        "g" => "gram"
        "l" => "litre"
        other => f"unknown '{other}'"

fn main():
    println(direction("north"), direction("n"), direction("south"), direction("s"), direction("nort"), direction("northh"), direction("North"))
    println(special(""), special("a"), special("a\0b"))
    println(unit("g"), unit("l"), unit("cup"))
    command: String = "quit"
    match command:
        QUIT:
            println("quit")
        _:
            println("other")

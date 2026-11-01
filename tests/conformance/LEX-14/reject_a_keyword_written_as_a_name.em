#$ test: compile-fail
#$ rules: LEX-14
#$ help: write `r#type` to use the word as a name
#$ help: write `r#class` to use the word as a name
## `[LEX-14]` — a keyword of the table written where a name is expected is `E0005`, "`type` is a
## keyword", with the help "write `r#type` to use the word as a name": a binding, an assignment's
## target, a field, a named argument, a field access, a parameter, a function's name and an operand.
## The parse goes on with the word as the name, so each use is one error and nothing else.

struct Token:
    class: int    #$ error[E0005]: `class` is a keyword

fn area(in: int) -> int:    #$ error[E0005]: `in` is a keyword
    return 2

fn while() -> int:    #$ error[E0005]: `while` is a keyword
    return 1

fn main():
    type = 3    #$ error[E0005]: `type` is a keyword
    type += 1    #$ error[E0005]: `type` is a keyword
    t = Token(class=2)    #$ error[E0005]: `class` is a keyword
    n = t.class    #$ error[E0005]: `class` is a keyword
    println(n + area(4))
    println(type)    #$ error[E0005]: `type` is a keyword

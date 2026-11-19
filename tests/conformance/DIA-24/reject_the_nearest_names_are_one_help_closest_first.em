#$ test: compile-fail
#$ rules: DIA-24, DIA-2
#$ error[E1010]: cannot find `counte` in this scope
#$ help: the nearest names are `count`, `counter` and `cout`
#$ error[E1010]: cannot find `coun` in this scope
#$ help: the nearest names are `count` and `cout`
#$ error[E1010]: cannot find `cnt` in this scope
#$ help: the nearest name is `cut`
#$ not-help: did you mean
# `[DIA-24]` — at most three names are offered, closest first and then by qualified name, in one help that
# states them as a fact of the program: "the nearest names are `count`, `counter` and `cout`", with a
# fix-it for the first. A name of up to 4 characters is offered others one edit apart only.

fn count() -> int:
    return 1

fn counter() -> int:
    return 2

fn cout() -> int:
    return 3

fn cut() -> int:
    return 4

fn main():
    println(counte(), coun(), cnt())

#$ test: compile-fail
#$ rules: BRW-18, EFF-1, EFF-24
# `[BRW-18]`, `[EFF-1]` — a call that may change the collection a loop walks is `E3020` at the call, and the
# message names what the called function does to it, by its summary, and through which function and line:
# "`note(ref mut xs)` (line 16) adds to `xs` (through `grow`, line 8) while this loop walks `xs` (line 15)".

fn grow(ref mut es: Array[int]):
    es.push(owns 1)

fn note(ref mut log: Array[int]):
    grow(ref mut log)

fn main():
    xs = [1]
    for x in xs:
        note(ref mut xs)    #$ error[E3020]: `note(ref mut xs)` (line 16) adds to `xs` (through `grow`, line 8) while this loop walks `xs` (line 15)
        println(x)

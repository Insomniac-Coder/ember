# Clause DIA-27.2: "No message asks for a word on a function's first line: what a function changes,
# reads and ends is worked out from its code (§X)"; `[EFF-24]`: "`changes`, `reads` or another
# clause after a function's brackets is `E0100`"
#$ test: compile-fail
#$ rules: DIA-27, EFF-24
# `[EFF-24]`'s message.

static mut FRAME: int = 0

fn tick() changes FRAME:    #$ error[E0100]: `changes FRAME` is not Ember: the compiler works out what `tick` changes from its body; write `fn tick():`
    FRAME += 1

fn main():
    tick()

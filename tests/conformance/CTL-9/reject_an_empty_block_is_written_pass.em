#$ test: compile-fail
#$ rules: CTL-9, LEX-9
# `[CTL-9]` — an empty block is written `pass`; a block with nothing in it is `E0004`.

fn nothing():
#$ error[E0004]: expected an indented block

fn main():
    nothing()

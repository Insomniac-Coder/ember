#$ test: run-pass
#$ rules: LEX-11a, LEX-8
#$ stdout: 1
## `[LEX-11a]` — a `##` comment on its own line inside a block attaches inside
## that block: it does not close or open one.

fn main():
    if true:
        ## a doc comment written inside the block
        println(1)

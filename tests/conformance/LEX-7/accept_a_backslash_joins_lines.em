#$ test: run-pass
#$ rules: LEX-7
#$ stdout: 3
## `[LEX-7]` — a backslash at the end of a physical line joins it to the next.

fn main():
    x = 1 + \
        2
    if x == 3 and \
            x > 0:
        println(x)

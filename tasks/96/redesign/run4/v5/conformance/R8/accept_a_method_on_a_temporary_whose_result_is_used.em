#$ test: run-pass
#$ rules: BRW-16
#$ stdout: 5
## design rules: R8
## R8 (d), verdict 13: `next` changes the temporary iterator, but its result is used, so the call is
## allowed. AN-10's first NEW program.

fn first_word(line: StrView) -> int:
    return line.split(" ").next().unwrap_or("").len()

fn main():
    println(first_word("hello world"))

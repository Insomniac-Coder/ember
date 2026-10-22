#$ test: compile-fail
#$ rules: CLI-19
#$ error[E0900]: a local generator is not supported yet
# A construct the specification defines and this compiler does not implement
# is rejected by name, never accepted with another meaning. (The generic local
# function this used is built, `[GRM-28]`.)

fn main():
    gen fn count(n: int) -> Generator[int]:
        for i in 0..n:
            yield i
    for v in count(2):
        println(v)

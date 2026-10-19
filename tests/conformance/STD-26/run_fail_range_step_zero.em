#$ test: run-fail
#$ rules: STD-26
#$ profiles: debug, development, release
#$ panics: range() step must not be zero

fn main():
    step = 0
    for i in range(0, 10, step):
        println(i)

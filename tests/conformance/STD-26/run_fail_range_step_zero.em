#$ test: run-fail
#$ rules: STD-26
#$ profiles: debug, release, shipping
#$ panics: range() step must not be zero

fn main():
    step = 0
    for i in range(0, 10, step):
        println(i)

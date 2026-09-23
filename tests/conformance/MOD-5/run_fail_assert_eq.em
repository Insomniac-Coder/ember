#$ test: run-fail
#$ rules: MOD-5
#$ profiles: debug, release, shipping
#$ panics: assertion failed: the two values are not equal

fn main():
    assert_eq(1 + 1, 3)

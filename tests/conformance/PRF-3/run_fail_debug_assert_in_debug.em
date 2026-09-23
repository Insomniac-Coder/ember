#$ test: run-fail
#$ rules: PRF-3
#$ profiles: debug
#$ panics: assertion failed

fn main():
    debug_assert(1 > 2)

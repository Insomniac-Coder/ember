#$ test: compile-fail
#$ rules: HASH-1, MOD-5

struct Probe:
    value: i32

extend Probe implements Hasher: #$ error[E1010]: cannot find interface `Hasher` in this scope
    pass

fn main():
    pass

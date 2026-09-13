#$ test: compile-fail
#$ rules: HASH-1, HASH-2, MOD-5

fn main():
    value: DefaultHasher #$ error[E1010]: cannot find type `DefaultHasher` in this scope

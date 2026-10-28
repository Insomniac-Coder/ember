#$ test: compile-fail
#$ rules: UNS-7, ATT-1
#$ profiles: debug
#$ error[E0104]: `@safety` requires one string obligation

@safety(42)
unsafe fn operation(x: i32) -> i32:
    return x

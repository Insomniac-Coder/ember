#$ test: compile-fail
#$ rules: FN-6b, TST-20, TST-21
#$ error[E0100]

fn main():
    value: @latebound i32 = 1

#$ test: compile-fail
#$ rules: TYP-25
#$ profiles: debug, release, shipping
#$ error[E2020]: no parameter `missing`
#$ error[E1030]: parameter `left` is given twice
#$ error[E2020]: positional arguments must come before named ones

fn pair(left: i32, right: i32):
    return

fn main():
    pair(missing=1, right=2)
    pair(left=1, left=2)
    pair(right=2, 1)

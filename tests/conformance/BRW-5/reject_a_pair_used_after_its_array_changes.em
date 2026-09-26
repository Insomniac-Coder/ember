#$ test: compile-fail
#$ rules: BRW-5, STD-15, LT-1
# `[BRW-5]` (ODR-068) — the pair borrows the array (`[LT-1]` rule 1): the
# array cannot grow while either reference lives, and the references cannot
# outlive the array. After the pair's last use the array is free again.

fn first_two(xs: Array[int]) -> ref mut int:
    tmp = xs.clone()
    match tmp.get_pair_mut(0, 1):
        Some((a, _)):
            return a #$ error[E3060]
        None:
            panic("short")

fn main():
    xs = [1, 2, 3]
    pair = xs.get_pair_mut(0, 1)
    xs.push(4) #$ error[E3022]
    match owned pair:
        Some((a, _)):
            a = 5
        None:
            pass
    xs.push(5)
    println(xs, first_two(xs))

#$ test: run-pass
#$ rules: STD-15, PHIL-5
#$ profiles: debug, release, shipping
#$ assert-c: !contains("vec_reserve_hint(")
#$ stdout:
#$ 8 16
# A program that asks any list for its `capacity()`, in any function, gets
# no hint for any loop: the capacity a list reports is how it grew, by
# doubling from four, as the program wrote it.

fn room(xs: Array[int]) -> int:
    return xs.capacity()

fn main():
    small: Array[int] = []
    for i in 0..5:
        small.push(i)
    large: Array[int] = []
    for i in 0..9:
        large.push(i)
    println(f"{room(small)} {room(large)}")

#$ test: compile-fail
#$ rules: STD-19, FN-2a
# `[STD-19]` (verdict 158) — a function written in a call to a standard-library function says
# `owns`: without it the call is `E3028`.

fn main():
    xs = [1, 2]
    xs.iter().for_each(fn(x: int): println(x))    #$ error[E3028]

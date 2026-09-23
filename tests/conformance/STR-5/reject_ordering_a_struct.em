#$ test: compile-fail
#$ rules: STR-5
#$ profiles: debug
#$ error[E2040]: `Point` does not implement `Ord`, which `<` needs
# `Ord` is never implicit: `==` works field-wise, `<` does not.

struct Point:
    x: int
    y: int

fn main():
    a = Point(x = 1, y = 2)
    println(a < a)

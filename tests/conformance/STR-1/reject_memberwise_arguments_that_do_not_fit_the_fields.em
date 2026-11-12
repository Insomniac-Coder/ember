#$ test: compile-fail
#$ rules: STR-1, TYP-25
# `[STR-1]` (N171) - arguments that do not fit a memberwise constructor's fields are `E2021`, as at any
# call; the message gives the constructor's first line.

struct Point:
    x: int
    y: int

fn main():
    a = Point(1)    #$ error[E2021]: `Point(1)` gives nothing for `y`, which has no default: `Point(x: int, y: int)`
    b = Point(1, 2, 3)    #$ error[E2021]: `Point(1, 2, 3)` gives 3 arguments, and `Point` takes 2: `Point(x: int, y: int)`
    c = Point(x=1, 2)    #$ error[E2021]: `Point(x=1, 2)` gives `2` after the named `x=1`, and positional arguments come first; write `Point(x=1, y=2)`
    d = Point(x=1, x=2)    #$ error[E2021]: `Point(x=1, x=2)` gives `x` twice
    e = Point(1, z=2)    #$ error[E2021]: `Point(1, z=2)` names `z`, and `Point` has no field of that name

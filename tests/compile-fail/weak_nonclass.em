#$ test: compile-fail
#$ rules: WK-1, OWN-13, WK-16
#$ error[E2119]: the i64 would end at the end of the line and the link would find nothing; hold it shared in a name first: write `i = shares 1` above, then `invalid = weakref i`
# `[OWN-13]` — `weakref` before a new value is `E2119`: a value that is not a class object lives on its
# own once it is held `shares`.

fn main():
    invalid = weakref 1

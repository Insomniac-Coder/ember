#$ test: compile-fail
#$ rules: DSJ-4, DIA-7a

# `[DSJ-4]` — two single-object references have no address range to compare:
# `E3095` (shape B11), whose help names the views it takes.

from std.mem import assert_disjoint

fn main():
    a = 1
    b = 2
    r = assert_disjoint(ref a, ref b)    #$ error[E3095]: `ref i64` is not a view, so its disjointness cannot be established

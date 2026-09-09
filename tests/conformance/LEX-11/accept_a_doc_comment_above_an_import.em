#$ test: run-pass
#$ rules: LEX-11
## "A `##` comment that is not followed by a declaration documents nothing and
## is **discarded in silence**: a comment never affects compilation, and that
## includes producing a warning." An import is not a declaration.

from std.math import min_i32

## This one does document something.
fn main():
    println(min_i32(1, 2))
#$ stdout: 1

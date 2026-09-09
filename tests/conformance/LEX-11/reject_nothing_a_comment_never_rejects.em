#$ test: parse-pass
#$ rules: LEX-11
## `[LEX-11]` has no rejection: a comment never affects compilation. The
## `[TST-4b]` waiver would apply, but the boundary case is worth pinning —
## a doc comment in every position that has no declaration after it.

## before an import
import std.math

## before nothing at all, at the end of a block
fn f():
    x = 1
    ## documents nothing
    print(x)

## before the end of the file

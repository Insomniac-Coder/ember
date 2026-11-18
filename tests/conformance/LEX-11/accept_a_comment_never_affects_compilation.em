#$ test: run-pass
#$ rules: LEX-11
#$ stdout: 3
## `[LEX-11]` - a comment never affects compilation, a diagnostic included: whatever it holds (an
## unterminated string, a keyword, brackets that do not close, an `E0100`-looking line), and wherever it
## stands (after a statement, between match arms, after the last item).

import std.io

## documents `f`, across a blank line

fn f(x: int) -> int:     ## trailing doc comment
    # a "string that never ends, [ ( { fn import class
    if x > 1:
        ## a doc comment that documents nothing
        return x
    ## nor does this one
    return 0

fn main():
    match f(3):
        ## between the arms
        3:
            println(3)
        # and a plain one
        _:
            println(0)

## documents nothing at the end

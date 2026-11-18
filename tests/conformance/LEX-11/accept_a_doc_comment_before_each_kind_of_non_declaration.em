#$ test: run-pass
#$ rules: LEX-11
#$ stdout: 4
## `[LEX-11]` - a `##` comment not followed by a declaration documents nothing and is discarded in
## silence: before an import, a statement, an expression, a `for`, a `defer`, and a dedent.

## before an import
import std.math

fn main():
    ## before an assignment
    total = 0
    ## before a loop
    for i in range(2):
        ## before a statement in the loop
        total += i
    ## before a call
    println(total + 3)
    ## before the end of the block

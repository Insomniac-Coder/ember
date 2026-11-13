#$ test: compile-fail
#$ rules: LEX-10
#$ error[E0100]: expected an expression
#$ error[E1010]: cannot find `hi`
#$ error[E0100]: expected an expression
## `[LEX-10]` - there are no block comments: `/* ... */` is not a comment.

fn main():
    /* hi */
    println(1)

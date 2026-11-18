#$ test: compile-fail
#$ rules: LEX-1a, LEX-11
## `[LEX-1a]` - a bidirectional format character in a comment is `E0012`: what a reviewer sees must be
## what the compiler reads. It is the one diagnostic a comment produces (`[LEX-11]`).

fn main():
    # a comment with a right-to-left override ‮ here    #$ error[E0012]: line 7 holds U+202E, which reorders how the line is shown
#$ help: `\u{202E}`
    println("ok")

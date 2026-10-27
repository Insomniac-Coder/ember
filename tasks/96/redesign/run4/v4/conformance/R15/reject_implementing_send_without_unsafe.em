#$ test: compile-fail
#$ rules: R15, R16
## R15: "`Send` is declared only by an `unsafe extend` author" (section 3's E3100 message, in this
## program's names; it replaces today's, which gave no line).

struct Raw:
    p: *mut int

extend Raw implements Send:    #$ error[E3100]: `Raw` holds a raw pointer, so the compiler cannot tell that another thread may have it; say that you vouch for it: write `unsafe extend Raw implements Send:` instead of line 9, with a `# SAFETY:` note above it
    pass

fn main():
    println(1)

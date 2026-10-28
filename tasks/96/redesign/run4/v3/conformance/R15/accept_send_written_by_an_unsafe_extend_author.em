#$ test: compile-pass
#$ rules: R15, R16
## R15: "`Send` is declared only by an `unsafe extend` author"; R16: before an implementation `unsafe`
## says "this code vouches", with a `# SAFETY:` note saying why.

struct Raw:
    p: *mut int

# SAFETY: a `Raw` is made only from a pointer the program hands over whole, and no other thread keeps it
unsafe extend Raw implements Send:
    pass

fn main():
    println(1)

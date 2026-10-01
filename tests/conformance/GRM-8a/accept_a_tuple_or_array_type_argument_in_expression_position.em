#$ test: run-pass
#$ rules: GRM-8a, GRM-8
#$ stdout: 16 32
#$ stdout: 24 8
# `[GRM-8a]` — inside `[ ]` in expression position, a type argument that does
# not start with a type-only word is parsed as an expression and read back as
# a type: a tuple of types and a fixed array too, not only a name or an
# instantiation (D-468).

fn main():
    println(mem.size_of[(int, int)](), mem.size_of[[int; 4]]())
    println(mem.size_of[[(i32, bool); 3]](), mem.align_of[(i8, i64)]())

#$ test: compile-fail
#$ rules: BRW-1, TYP-14, BRW-15
# "while shared borrows are live the owner may read (and copy) but not write". `=` with a value on a
# name holding a `ref` writes what it refers to, as on a `ref mut` name, so through a read-only one it
# is `[BRW-15]`'s refusal, with the line to write (M5 INDEX Q3).
#
# This compiled once. It emitted `(*_2) = 99` and was caught by **clang**
# ("read-only variable is not assignable"), never by the compiler, so
# aliasing-XOR-mutability was being upheld by the backend's `const` rather
# than by the language.

fn main():
    x: i32 = 1
    r: ref i32 = ref x
    r = 99                 #$ error[E3023]: `r = 99` changes `x`, but `r` only reads it; to change `x` through `r`, write `r = ref mut x` in place of line 14
    println(x)

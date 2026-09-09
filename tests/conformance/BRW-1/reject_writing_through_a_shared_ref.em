#$ test: compile-fail
#$ rules: BRW-1, TYP-14
# "while shared borrows are live the owner may read (and copy) but not write".
# ADR-010 settled that a reference local is not re-seatable — `m = ref mut y`
# writes *through* `m` — which is sound for `ref mut` and is exactly the
# forbidden write for a shared `ref`.
#
# This compiled until now. It emitted `(*_2) = 99` and was caught by **clang**
# ("read-only variable is not assignable"), never by the compiler, so
# aliasing-XOR-mutability was being upheld by the backend's `const` rather
# than by the language.

fn main():
    x: i32 = 1
    r: ref i32 = ref x
    r = 99                 #$ error[E3021]: cannot write through a shared reference
    println(x)

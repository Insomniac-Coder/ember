#$ test: compile-fail
#$ rules: UNS-10
#$ help: if the value is genuinely shared, use `RefCell[T]` for runtime-checked borrows
#$ not-help: UnsafeCell
# `[UNS-10]` — diagnostics never suggest `UnsafeCell`: a change through a read-only parameter is
# offered `ref mut`, a `RefCell` or a class, never the unchecked cell.

struct Hits:
    n: int

fn bump(h: Hits):
    h.n += 1    #$ error[E3023]: `bump` sets `h.n`, but `h` only reads the caller's hits

fn main():
    h = Hits(n=0)
    bump(h)
    println(h.n)

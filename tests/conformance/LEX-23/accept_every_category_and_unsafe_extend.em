#$ test: run-pass
#$ rules: LEX-23, UNS-8
#$ stdout: 1
#$ stdout: 2
#$ stdout: 3
#$ stdout: 4
#$ stdout: 5
#$ stdout: 6
## `[LEX-23]` — `SAFETY:` and `SAFETY(<category>):` with a category of `ffi`, `layout`, `aliasing`,
## `intrinsic`, `performance` or `uninit`, on the line before an `unsafe:` block (or at the end of its
## line), and before an `unsafe extend` of `Send` or `Sync`, is the construct's safety note: nothing warns.

struct Cell:
    n: int = 0

# SAFETY(layout): a Cell holds one integer, which any thread may read
unsafe extend Cell implements Send:
    pass

@safety("any call is sound")
unsafe fn one(n: int) -> int:
    return n

fn main():
    # SAFETY(ffi): nothing foreign is called
    unsafe:
        println(one(1))
    # SAFETY(layout): nothing is laid out
    unsafe:
        println(one(2))
    # SAFETY(aliasing): nothing is aliased
    unsafe:
        println(one(3))
    # SAFETY(intrinsic): nothing is an intrinsic
    unsafe:
        println(one(4))
    unsafe:  # SAFETY(performance): at the end of the line
        println(one(5))
    unsafe:  # SAFETY(uninit): also at the end
        println(one(6))

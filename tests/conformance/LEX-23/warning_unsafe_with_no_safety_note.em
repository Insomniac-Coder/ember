#$ test: run-pass
#$ rules: LEX-23, UNS-8
#$ stdout: 6
#$ stdout: 7
## `[UNS-8]` — an `unsafe:` block or an `unsafe` implementation (here `unsafe extern`) with no safety
## note is `W3012`; a `SAFETY(...)` whose category is not one of `ffi`, `layout`, `aliasing`,
## `intrinsic`, `performance` or `uninit` is no note (D-584). An `unsafe fn` says what its callers make
## true with `@safety` instead.

unsafe extern "C":    #$ warning[W3012]: this `unsafe` has no safety note
    @safety("callers pass what the C function `abs` requires")
    unsafe fn abs(x: i32) -> i32

@safety("nothing: abs takes any i32")
unsafe fn bare(x: i32) -> i32:
    return abs(x)

fn main():
    unsafe:    #$ warning[W3012]: this `unsafe` has no safety note
        println(bare(-6))
    # SAFETY(weird): not a category
    unsafe:    #$ warning[W3012]: this `unsafe` has no safety note
        println(bare(-7))

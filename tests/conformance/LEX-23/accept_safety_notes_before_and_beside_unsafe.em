#$ test: run-pass
#$ rules: LEX-23, UNS-8
#$ stdout: 4
#$ stdout: 5
## `[LEX-23]` — a comment beginning `SAFETY:` or `SAFETY(<category>):`, alone
## on the line before an `unsafe:` block or `unsafe` implementation or at the
## end of its first line, is that construct's safety note; with one each, and
## each `unsafe fn` saying what its callers make true with `@safety`, nothing warns.

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @safety("callers pass what the C function `abs` requires")
    fn abs(x: i32) -> i32

@safety("abs takes any i32 and touches no memory")
unsafe fn magnitude(x: i32) -> i32:
    return abs(x)

@safety("as magnitude")
unsafe fn trailing(x: i32) -> i32:
    return abs(x)

fn main():
    unsafe:  # SAFETY(ffi): magnitude's note covers its call
        println(magnitude(-4))
    # SAFETY: the same
    unsafe:
        println(trailing(-5))

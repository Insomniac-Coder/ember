#$ test: run-pass
#$ rules: LEX-23, UNS-8
#$ stdout: 4
#$ stdout: 5
## `[LEX-23]` — a comment beginning `SAFETY:` or `SAFETY(<category>):`, alone
## on the line before an `unsafe:` block or `unsafe fn` or at the end of its
## first line, is that construct's safety note; with one each, nothing warns.

unsafe extern "C":
    fn abs(x: i32) -> i32

# SAFETY(ffi): abs takes any i32 and touches no memory
unsafe fn magnitude(x: i32) -> i32:
    return abs(x)

unsafe fn trailing(x: i32) -> i32:  # SAFETY: as magnitude
    return abs(x)

fn main():
    unsafe:  # SAFETY(ffi): magnitude's note covers its call
        println(magnitude(-4))
    # SAFETY: the same
    unsafe:
        println(trailing(-5))

#$ test: run-pass
#$ command: lint
#$ rules: LNT-6, FFI-1, FFI-11
#$ stdout: 5 3
## `[LNT-6]` — `L5001`: an `unsafe extern` declaration that takes or returns a raw pointer and
## states no contract: each call needs `unsafe`, and nothing is known of what C does with the
## pointer. A declaration with a contract, or with no pointer, is no lint.

unsafe extern "C":
    fn strlen(text: *u8) -> usize  #$ warning[L5001]: `strlen` is an `unsafe extern` declaration with no contract
    @ffi(param(text, borrowed, nul_terminated), link_name="strlen")
    safe fn c_length(text: cstr) -> usize
    fn abs(n: i32) -> i32

fn main():
    println(c_length(c"hello"), c_length(c"abc"))

#$ test: run-pass
#$ rules: FFI-10, FFI-9
#$ profiles: debug, release
#$ stdout: 42
#$ stdout: 9000000000
#$ stdout: 17
#$ assert-c: contains("EMBER_CC_SYSTEM abs(")
#$ assert-c: contains("EMBER_CC_FASTCALL imaxabs(")
# `[FFI-10]`, `[FFI-9]` — a declaration block names its calling convention:
# `"system"` is `stdcall` on 32-bit Windows and C elsewhere, and on x86-64
# `"fastcall"` is the one convention there, so the C library's functions are
# reached through both. A function value keeps the convention in its type.

unsafe extern "system":
    safe fn abs(x: i32) -> i32

unsafe extern "fastcall":
    safe fn imaxabs(x: i64) -> i64

fn main():
    println(abs(-42))
    println(imaxabs(-9000000000))
    absolute = abs
    println(absolute(-17))

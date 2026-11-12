#$ test: run-pass
#$ rules: FFI-10, TYP-1
#$ profiles: development, release
#$ stdout: 5 7
#$ assert-c: contains("extern int64_t em_")
#$ assert-c: contains("__asm__(")
# `[FFI-10]` — a hand declaration of a C library function whose parameter is a C `long long`
# (D-693): `llabs`, which `<stdlib.h>` declares `long long llabs(long long)`. Ember's `i64` is
# `int64_t`, which is `long` on LP64 Linux, so a C prototype named `llabs` taking an `int64_t` made gcc
# stop with "conflicting types". The compiler names its own declaration apart from every system
# header's and binds it to the C symbol with an assembler label, so a header's prototype and the
# declaration never meet, on every platform and C compiler. (Checked with gcc 15.2 on Linux.)

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(effects=[])
    safe fn llabs(x: i64) -> i64

fn main():
    println(llabs(-5), llabs(7))

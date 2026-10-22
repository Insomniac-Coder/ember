#$ test: run-pass
#$ rules: FFI-10, STA-1, FFI-6, BLD-FFI-1
#$ profiles: debug, development, release
#$ stdout: 7 7 7
# Part XVI's import table and `[STA-1]`, by the owner's ruling of 2026-10-21 (ODR-163): a foreign
# static read and written only in `unsafe` may hold what is not `Sync`, such as a pointer. A C
# header's global holding a pointer imports as any other (`c.cursor`), and a hand-declared one is
# accepted too; the programmer, not the checker, answers for sharing them between threads.

import c "./cursor.c"

unsafe extern "C":
    static mut foreign_cursor: *mut i32
    static foreign_base: *i32

fn main():
    unsafe:  # SAFETY(ffi): one thread; cursor points at C's live `slot`
        p = c.cursor
        foreign_cursor = p
        b = c.banner
        c.banner = b
        println(p.read(), foreign_cursor.read(), foreign_base.read())

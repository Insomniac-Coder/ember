#$ test: run-pass
#$ rules: STA-1, THR-8, THR-9
#$ stdout: 1
# `[STA-1]`, `[THR-8]` — a `static` is `Sync`, and a `Mutex[T]` is when `T` is `Send`; a type that holds
# a raw pointer is `Send` where the program says so with `unsafe extend` (D-834: the statics were
# checked before the `unsafe extend` of the same program was recorded, so this was `E7002`).

from std.sync import Mutex

struct Handle:
    address: *mut int

unsafe extend Handle implements Send:  # SAFETY: the address is never read, only compared
    pass

static HANDLES: Mutex[Array[Handle]] = Mutex([])

fn main():
    with all = HANDLES.lock():
        println(all.len() + 1)

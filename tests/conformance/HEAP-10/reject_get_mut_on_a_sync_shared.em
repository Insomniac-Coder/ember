#$ test: compile-fail
#$ rules: HEAP-10
#$ error[E1010]: `SyncShared[i64]` has no method named `get_mut`
# `[HEAP-10]` — a `SyncShared` offers `get() -> ref T` only.

from std.sync import SyncShared

fn main():
    s = SyncShared(5)
    s.get_mut()

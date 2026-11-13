#$ test: compile-fail
#$ rules: STR-3, DRV-1, OWN-7
## `[STR-3]` - `@derive(Copy)` is `E2080` if a field is not `Copy`: an owned `String` or a `MutSpan` (a class object's
## handle is `Copy` in this compiler, `[OWN-7]`: STR-3's class clause waits for M4's A1-9).

@derive(Copy)
struct HoldsText:
    name: String    #$ error[E2080]: field `name` has type `String`, which is not Copy

@derive(Copy)
struct HoldsView:
    m: MutSpan[int]    #$ error[E2080]: field `m` has type `MutSpan[i64]`, which is not Copy

fn main():
    pass

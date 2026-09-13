#$ test: run-pass
#$ rules: TYP-18, ARN-1

# Explicit method arguments must remain attached to the receiver through
# parsing and resolution. Arena.alloc is compiler-known today, but follows the
# same `[TYP-18]` argument-selection rule as a source-declared generic call.
fn main():
    arena = Arena.with_capacity(16)
    stored: ref mut i64 = arena.alloc[i64](41)
    println(stored)
#$ stdout: 41

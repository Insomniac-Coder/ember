#$ test: run-pass
#$ rules: EFF-3, FFI-10, DET-2
#$ profiles: debug, development
#$ stdout: 5 7
#$ stdout: 9
# `[EFF-3]` — a foreign function carries the effects its contract states,
# `FFI` at least: `@ffi(effects=[])` says `abs` and `imaxabs` allocate, lock, wait
# and print nothing, so contracted callers may call them. `@ffi(deterministic)`
# says the result is the same on every run, so a `@deterministic` caller may
# too (`[DET-2]`); `FFI` is no contract's concern.

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(effects=[])
    safe fn abs(x: i32) -> i32
    @ffi(effects=[], deterministic)
    safe fn imaxabs(x: i64) -> i64

@noalloc @nolock @noblock @noio @nosync @nopanic(explicit)
fn magnitude(x: i32) -> i32:
    return abs(x)

@deterministic
fn distance(a: i64, b: i64) -> i64:
    return imaxabs(a - b)

fn main():
    println(magnitude(-5), magnitude(7))
    println(distance(3, 12))

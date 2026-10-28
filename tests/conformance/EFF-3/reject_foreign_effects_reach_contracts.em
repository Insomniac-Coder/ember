#$ test: compile-fail
#$ rules: EFF-3, FFI-10, EFF-6, DET-2
#$ error[E4001]: `@noalloc` function `copy_text` reaches an allocation
#$ error[E4001]: `@noalloc` function `undeclared` reaches an allocation
#$ error[E4041]: `@noio` function `undeclared` reaches I/O
#$ error[E4070]: `@deterministic` function `steady` reaches a nondeterministic operation
# `[EFF-3]` — a foreign function has the effects its contract states, and
# every effect when it states none, so a contract over its caller is checked
# against them (`[EFF-6]`); without `@ffi(deterministic)` it is `Nondet`.

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(effects=[Alloc, Panic])
    fn strdup(text: *u8) -> *u8
    safe fn rand() -> i32
    @ffi(effects=[])
    safe fn abs(x: i32) -> i32

@noalloc
fn copy_text(text: *u8) -> *u8:
    unsafe:  # SAFETY(ffi): the caller passes a NUL-terminated string
        return strdup(text)

@noalloc @noio
fn undeclared() -> i32:
    return rand()

@deterministic
fn steady(x: i32) -> i32:
    return abs(x)

fn main():
    println(undeclared(), steady(-1))

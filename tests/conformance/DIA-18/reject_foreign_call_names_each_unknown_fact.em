#$ test: compile-fail
#$ rules: DIA-18, FFI-2, FFI-10, FFI-11c
#$ profiles: debug
#$ help: state the facts on its declaration: `safe fn abs`
#$ help: `@ffi(param(data, TODO(ownership), TODO(count), TODO(nullable)), result(TODO(ownership), TODO(count), TODO(nullable), TODO(lifetime)))` over `safe fn find`
#$ help: `@ffi(param(text, borrowed, TODO(count)))` over `safe fn measure`
# `[DIA-18]` — a foreign call refused for its contract's unknown facts names
# each one and prints the declaration line that states them, every unknown
# fact a `TODO(…)` word to replace (`[FFI-11c]`): the line compiles on the
# unsafe declaration as printed.

unsafe extern "C":
    fn abs(x: i32) -> i32
    fn find(data: *u8, key: u8) -> *u8
    @ffi(param(text, borrowed, TODO(count)))
    fn measure(text: *u8) -> usize

fn main():
    println(abs(-42))    #$ error[E5002]: unknown: `abs` is not asserted safe to call (`safe fn`)
    found = find(null[*u8](), 3)    #$ error[E5002]: unknown: `data`: its ownership, count and nullability
    println(measure(null[*u8]()))    #$ error[E5002]: unknown: `text`: its count

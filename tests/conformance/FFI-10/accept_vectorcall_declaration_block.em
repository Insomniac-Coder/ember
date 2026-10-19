#$ test: compile-pass
#$ rules: FFI-10, FFI-9
#$ assert-c: contains("#define EMBER_CC_VECTORCALL __vectorcall")
# `[FFI-10]`, `[FFI-9]` — `extern "vectorcall"` declares functions with x86's
# vector calling convention (MSVC's `__vectorcall`, Clang's attribute). A C
# compiler or target without it stops the C build naming the convention, never
# calling with another one. The milestone
# `vectorcall_declarations_compile_with_the_convention` compiles a call.

unsafe extern "vectorcall":
    safe fn scale_vector(x: f64, factor: f64) -> f64

fn main():
    pass

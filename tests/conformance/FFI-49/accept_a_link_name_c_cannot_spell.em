#$ test: run-pass
#$ rules: FFI-49
#$ profiles: debug, development, release
#$ stdout: 42
#$ assert-c: contains("__asm__(")
#$ assert-c: contains("/ALTERNATENAME:")
# `[FFI-49]` — `@ffi(link_name="sym")` binds a declaration to a differently named symbol, one C
# cannot spell among them: the C declares it under an alias of its own and binds the alias to the
# symbol (an assembler label; MSVC's linker resolving the alias). `import c "./weird.c"` compiles the
# C file that defines it.

import c "./weird.c"

unsafe extern "C":
    @ffi(link_name="weird$twice")
    safe fn twice(x: i32) -> i32

fn main():
    println(twice(21))

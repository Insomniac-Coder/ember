#$ test: run-pass
#$ rules: FFI-26, MNG-2
#$ profiles: debug, development
#$ stdout: 7
#$ assert-c: contains("em_0x67616d652d757064617465(void) __asm__(EMBER_LABEL_PREFIX(__USER_LABEL_PREFIX__)")
#$ assert-c: contains("/ALTERNATENAME:game-update=em_0x67616d652d757064617465")
# `[FFI-26]`, `[MNG-2]` — `@export("name")` sets the symbol exactly, even one C cannot spell:
# the function keeps a C name of its own (`em_0x` and the symbol in hex), and its object-file
# symbol is `game-update` (an assembler label; with MSVC, a linker alias of that C name).
# Ember code calls it as any function.

@export("game-update")
fn update() -> i32:
    return 7

fn main():
    println(update())

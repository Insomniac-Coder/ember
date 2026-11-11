#$ test: run-pass
#$ rules: STA-2, STA-1
#$ profiles: debug, development, release
#$ assert-c: !contains("__attribute__((constructor))")
#$ stdout: 42
# `[STA-2]` — statics are initialised at compile time or on first access, never by a global
# constructor.

static ANSWER: int = 6 * 7

fn main():
    println(ANSWER)

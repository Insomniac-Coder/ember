#! language "0.9.9"
#$ test: compile-fail
#$ rules: VER-8, VER-1
#$ error[E0006]: language version `0.9.9` is not the current language
#$ help: delete the line
# `[VER-8]`, `[VER-1]` — the language version is this document's, 0.9.10, which moves
# independently of the compiler's version and the runtime's ABI version: 0.9.9 is the previous
# language, and before 1.0 a file can name only the current one (D-591: the compiler took 0.9.9
# for the current language and refused 0.9.10).

fn main():
    println(1)

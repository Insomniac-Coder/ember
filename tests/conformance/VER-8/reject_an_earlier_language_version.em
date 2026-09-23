#! language "0.9.8"
#$ test: compile-fail
#$ rules: VER-8
#$ profiles: debug
#$ error[E0006]: language version `0.9.8` is not the current language
#$ help: delete the line
# Before 1.0 a file does not select a language version.

fn main():
    println(1)

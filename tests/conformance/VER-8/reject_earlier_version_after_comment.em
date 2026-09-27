# A directive before imports still applies after a leading comment.
#! language "0.9.8"
#$ test: compile-fail
#$ rules: VER-8, GRM-37
#$ error[E0006]: language version `0.9.8` is not the current language

fn main():
    pass

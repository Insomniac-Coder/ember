#$ test: compile-fail
#$ rules: TXT-3
## `[TXT-3]` - a `c"..."` literal containing a NUL is `E5064`, however the NUL is written.

fn main():
    c = c"a\x00b"    #$ error[E5064]: a C string literal may not contain a NUL byte
    println(1)

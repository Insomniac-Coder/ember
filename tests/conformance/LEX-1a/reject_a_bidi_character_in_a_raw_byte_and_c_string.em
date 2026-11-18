#$ test: compile-fail
#$ rules: LEX-1a
## `[LEX-1a]` - a raw string, a byte string and a C string literal are source text too: each is E0012.

fn main():
    a = r"raw ‪ here"    #$ error[E0012]: line 6 holds U+202A
    b = b"byte ‫ here"    #$ error[E0012]: line 7 holds U+202B
#$ error[E0009]: ASCII only
    c = c"cstr ⁩ here"    #$ error[E0012]: line 9 holds U+2069
#$ error[E0009]: ASCII only
    println(a)

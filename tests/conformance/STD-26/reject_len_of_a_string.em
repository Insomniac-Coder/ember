#$ test: compile-fail
#$ rules: STD-26
#$ profiles: debug
#$ error[E2073]: `len` of a string
#$ help: `s.char_count()` for Python's count of characters
# Python counts characters and Ember's `s.len()` counts bytes, so `len(s)` asks
# which is meant.

fn main():
    name = "héllo"
    println(len(name))

#$ test: compile-fail
#$ rules: TXT-10
# `[TXT-10]` (ODR-029) — `parse` is told what to read, and reads integers,
# floats, `bool` and `char`.

fn main():
    a = "1".parse()                 #$ error[E2060]: cannot tell what to parse the text into
    b = "1".parse[Array[int]]()     #$ error[E2040]: `parse` reads integers, floats, `bool` and `char`, not `Array[i64]`

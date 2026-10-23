#$ test: run-pass
#$ rules: CG-C-7, MNG-3
#$ profiles: debug, release
#$ stdout: 18
#$ 21
#$ 1
#$ 2
#$ 10
#$ assert-c: contains("ember_span values")
#$ assert-c: contains("    int64_t running_sum;")
#$ assert-c: contains("    int64_t gr_u00F6__u00DF_e;")
#$ assert-c: contains("    em_coll_Array_i64 numbers;")
#$ assert-c: contains("    int64_t register_")
#$ assert-c: contains("    int64_t size_t_")
#$ assert-c: !contains("    int64_t size_t;")
#$ assert-c: contains("    int64_t i;")
#$ assert-c: contains("    int64_t i_")
# `[CG-C-7]`: locals keep their Ember names in the C, so a debugger shows them: a parameter
# (`values`; a view parameter Windows passes as two words is rebuilt under its name at entry), a
# local (`running_sum`, `numbers`, whose C type is its collection's own struct, `[CG-C-9]`), a
# non-ASCII name transliterated as `[MNG-3]` says (`größe` is `gr_u00F6__u00DF_e`). A name C would read as something else keeps the name with the
# local's number after it: a C keyword (`register`), or a name the function's C already uses
# (`size_t`, the type of its indices). Two locals of one name in one function (each loop's `i`)
# are the name and the name with its number.

fn total(values: Span[i64]) -> i64:
    running_sum = 0
    for value in values:
        running_sum += value
    return running_sum

fn main():
    größe = 3
    register = 4
    size_t = 5
    numbers = [1, 2, 3]
    println(total(numbers) + größe + register + size_t)
    println(total(numbers) + größe * 5)
    for i in 0..2:
        println(numbers[i])
    for i in 0..1:
        println(numbers[i] * 10)

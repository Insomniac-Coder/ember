#$ test: run-pass
#$ rules: GRM-2, FN-8
#$ profiles: debug, release, shipping
#$ stdout: 14
#$ big
# The entry file's statements at file scope form, in source order, the body of
# an implicit `fn main()`; items anywhere in the file are declared as usual,
# so a function declared after the statements that call it is found.

fn square(x: int) -> int:
    return x * x

total = 0
for i in 0..limit(4):
    total += square(i)
println(total)

fn limit(n: int) -> int:
    return n

if total > 10:
    println("big")

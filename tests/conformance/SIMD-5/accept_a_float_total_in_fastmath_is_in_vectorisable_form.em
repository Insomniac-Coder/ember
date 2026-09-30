#$ test: run-pass
#$ rules: SIMD-5, SIMD-7, TYP-9
#$ profiles: debug, release, shipping
#$ stdout: 0 51 117 30.0
#$ stdout: 0 51 117 30.0
#$ assert-c-count: contains("((uint64_t)((int64_t*)") == 2
# `[SIMD-5]` — a floating-point running total keeps a loop out of vectorisable
# form "unless the function is `@fastmath`": its C file lets the C compiler
# reorder the total (`[CG-C-11]`). So in `fast` the loop is in vectorisable
# form and its `+` check is grouped (`[SIMD-7]`), and in `strict`, the same
# loop, the check stays one per operation.

# Lent to a function, a list holds values the range facts cannot know
# (`[RNG-4]`), so the checks this test looks at stay.
fn unknown[T](mut xs: Array[T]):
    pass

@fastmath
fn fast():
    a: Array[int] = []
    b: Array[int] = []
    out: Array[int] = []
    fs: Array[f64] = []
    for i in 0..40:
        a.push(i)
        b.push(2 * i)
        out.push(0)
        fs.push(0.75)
    unknown(a)
    unknown(b)
    total = 0.0
    for i in 0..40:
        out[i] = a[i] + b[i]
        total += fs[i]
    println(out[0], out[17], out[39], total)

fn strict():
    a: Array[int] = []
    b: Array[int] = []
    out: Array[int] = []
    fs: Array[f64] = []
    for i in 0..40:
        a.push(i)
        b.push(2 * i)
        out.push(0)
        fs.push(0.75)
    unknown(a)
    unknown(b)
    total = 0.0
    for i in 0..40:
        out[i] = a[i] + b[i]
        total += fs[i]
    println(out[0], out[17], out[39], total)

fn main():
    fast()
    strict()

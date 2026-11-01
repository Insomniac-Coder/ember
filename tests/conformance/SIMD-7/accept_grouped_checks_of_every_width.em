#$ test: run-pass
#$ rules: SIMD-7, SIMD-5, OPT-2
#$ profiles: debug, development, release
#$ stdout: 2147483500 2147483599 155 254 1000 901 -32600 -32699
#$ assert-c-count: contains(" >> 63ULL;") == 8
# Grouped checks on `i32`, `u8`, `u64` and `i16` lists, close to each type's
# limits but never past them: each group finds no overflow and every result
# is exact. 100 iterations: three groups, then four checked one at a time.

# Lent to a function, a list holds values the range facts cannot know
# (`[RNG-4]`), so the checks this test looks at stay.
[T]
fn unknown(ref mut xs: Array[T]):
    pass

fn main():
    a: Array[i32] = []
    b: Array[i32] = []
    c: Array[u8] = []
    d: Array[u8] = []
    e: Array[u64] = []
    f: Array[u64] = []
    g: Array[i16] = []
    h: Array[i16] = []
    o1: Array[i32] = []
    o2: Array[u8] = []
    o3: Array[u64] = []
    o4: Array[i16] = []
    for i in 0..100:
        a.push(owns (2147483500 as i32))
        b.push(owns (i as i32))
        c.push(owns (155 as u8))
        d.push(owns (i as u8))
        e.push(owns ((i + 1000) as u64))
        f.push(owns ((2 * i) as u64))
        g.push(owns (-32600 as i16))
        h.push(owns (i as i16))
        o1.push(owns (0 as i32))
        o2.push(owns (0 as u8))
        o3.push(owns (0 as u64))
        o4.push(owns (0 as i16))
    unknown(ref mut a)
    unknown(ref mut b)
    unknown(ref mut c)
    unknown(ref mut d)
    unknown(ref mut e)
    unknown(ref mut f)
    unknown(ref mut g)
    unknown(ref mut h)
    for i in 0..100:
        o1[i] = a[i] + b[i]
    for i in 0..100:
        o2[i] = c[i] + d[i]
    for i in 0..100:
        o3[i] = e[i] - f[i]
    for i in 0..100:
        o4[i] = g[i] - h[i]
    println(o1[0], o1[99], o2[0], o2[99], o3[0], o3[99], o4[0], o4[99])

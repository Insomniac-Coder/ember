#$ test: run-pass
#$ rules: LT-14, LT-16, LT-19, LT-20, LT-24, TST-17
#$ stdout: 1
#$ stdout: 2
#$ stdout: 3
#$ stdout: 4
#$ stdout: 5
#$ stdout: 6
#$ stdout: 7
#$ stdout: 8

@view
struct Eight:
    a: Span[i32]
    b: Span[i32]
    c: Span[i32]
    d: Span[i32]
    e: Span[i32]
    f: Span[i32]
    g: Span[i32]
    h: Span[i32]

fn main():
    a: Array[i32] = Array[i32]()
    b: Array[i32] = Array[i32]()
    c: Array[i32] = Array[i32]()
    d: Array[i32] = Array[i32]()
    e: Array[i32] = Array[i32]()
    f: Array[i32] = Array[i32]()
    g: Array[i32] = Array[i32]()
    h: Array[i32] = Array[i32]()
    a.push(1)
    b.push(2)
    c.push(3)
    d.push(4)
    e.push(5)
    f.push(6)
    g.push(7)
    h.push(8)
    views = Eight(a.as_span(), b.as_span(), c.as_span(), d.as_span(),
                  e.as_span(), f.as_span(), g.as_span(), h.as_span())
    println(views.a[0])
    a.push(11)
    println(views.b[0])
    b.push(12)
    println(views.c[0])
    c.push(13)
    println(views.d[0])
    d.push(14)
    println(views.e[0])
    e.push(15)
    println(views.f[0])
    f.push(16)
    println(views.g[0])
    g.push(17)
    println(views.h[0])
    h.push(18)

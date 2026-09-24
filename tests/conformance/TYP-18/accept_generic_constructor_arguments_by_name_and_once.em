#$ test: run-pass
#$ rules: TYP-18, STR-1
#$ stdout:
#$ true 1.5
#$ 42
#$ 1 7 3 4
# `[TYP-18]` — a generic struct's type arguments come from the field each
# argument names, in any order, and each argument is checked once: a lambda
# written in the constructor is one closure (D-236). An expected type still
# gives a literal its type.

struct P[A, B]:
    a: A
    b: B

struct K[F]:
    f: F
    n: int

p = P(b=1.5, a=true)
println(p.a, p.b)
n = 40
k = K(f=fn(x: int) => x + n, n=2)
println(k.f(k.n))
x: u8 = 7
q = P(1, x)
r: P[u8, i16] = P(3, 4)
println(q.a, q.b, r.a, r.b)

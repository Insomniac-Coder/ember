#$ test: run-pass
#$ rules: STR-2, STR-1
#$ stdout:
#$ 3 5 3 1
#$ 1 0 4 x
#$ 7 0 1 2.5
#$ 99
#$ 10 20
# `[STR-2]` — a field default is any expression, evaluated at each
# construction that omits the field, in field order, and may allocate. Each
# construction gets its own `Array`. A default reads names as its declaration
# does: a constant of its module, never the constructing function's locals.
# D-238: a zero stood in for every default, and C spread an aggregate's `0`
# over the fields after it (`S(n=4)` put the 4 into `items`).

const START: int = 10

struct T:
    n: int = 3
    m: int = 5

struct S:
    items: Array[int] = []
    n: int = 3
    name: String = "x"

struct W[U]:
    v: U
    n: int = 7
    tags: Array[String] = []

struct C:
    first: int = START
    second: int = START * 2

fn make() -> C:
    START = 99
    println(START)
    return C()

t = T()
u = T(m=1)
println(t.n, t.m, u.n, u.m)
a = S()
b = S(n=4)
a.items.push(1)
println(len(a.items), len(b.items), b.n, a.name)
w = W(v=true)
x = W(v=2.5, n=1)
println(w.n, len(w.tags), x.n, x.v)
c = make()
println(c.first, c.second)

#$ test: run-pass
#$ rules: GRM-5, EXP-2
#$ stdout: rhs
#$ stdout: 10
#$ stdout: 20
#$ stdout: 2
#$ stdout: 1
#$ stdout: 30
#$ stdout: 40
#$ stdout: 50
#$ stdout: 60
#$ stdout: 70
#$ stdout: 80
#$ stdout: 90

struct Pair:
    pub first: i32
    pub second: i32

fn make_pair() -> (i32, i32):
    println("rhs")
    return (10, 20)

fn main():
    # Fresh targets are declarations and the RHS is evaluated once.
    a, b = make_pair()
    println(a)
    println(b)

    # Existing targets are assignments, with tuple-swap semantics.
    x = 1
    y = 2
    x, y = (y, x)
    println(x)
    println(y)

    # Struct fields are destructured in declaration order.
    c, d = Pair(30, 40)
    println(c)
    println(d)

    # Parenthesised and nested target lists retain their field paths.
    (e, f) = (50, 60)
    g, (h, i) = (70, (80, 90))
    println(e)
    println(f)
    println(g)
    println(h)
    println(i)

    # A wholly discarded aggregate is still evaluated once and then ends.
    _, _ = (100, 200)

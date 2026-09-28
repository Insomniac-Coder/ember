#$ test: run-pass
#$ rules: CTL-1, CTL-3, CTL-4, SPN-5
#$ profiles: debug, release, shipping
#$ assert-c-count: contains(".index = (") == 1
#$ stdout:
#$ 510
#$ [6, 7, 8]
#$ found 7
#$ no 9
#$ 3
#$ 7
# `[CTL-1]` — a `for` over a view's element iterator (`iter()`, `iter_mut()`)
# yields `&source[i]` from the iterator's cursor to the view's length, as
# `next` would, and the loop is a counted one with no `Option` in it
# (`[CTL-3]`'s spirit). `iter_mut` items change the list in place; an
# iterator already moved on starts where it stands (its one `next` is the
# only write of a cursor in the C); `break` skips the `else` (`[CTL-4]`);
# a tuple pattern binds through the borrowed item.

struct P:
    x: int
    y: int

fn main():
    ps: Array[P] = []
    for i in 0..5:
        ps.push(P(i, 10 * i))
    for p in ps.iter_mut():
        p.x += 100
    total = 0
    for p in ps.iter():
        total += p.x
    println(total)

    xs: Array[int] = [5, 6, 7, 8]
    it = xs.iter()
    it.next()
    seen: Array[int] = []
    for x in it:
        seen.push(x)
    println(seen)

    for x in xs.iter():
        if x == 7:
            println("found", x)
            break
    else:
        println("none")
    for x in xs.iter():
        if x == 9:
            break
    else:
        println("no 9")

    pairs: Array[(int, int)] = [(1, 2), (3, 4)]
    for (a, b) in pairs.iter():
        println(a + b)

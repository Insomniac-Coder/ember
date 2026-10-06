#$ test: run-pass
#$ rules: ALC-1
#$ stdout: 200 19900
#$ stdout: 100 abcdefghijklmnopqrstuvwxyabcdefghijklmnopqrstuvwxyabcdefghijklmnopqrstuvwxyabcdefghijklmnopqrstuvwxy
#$ stdout: 20 195 5
#$ stdout: 30 32
#$ stdout: 99
#$ stdout: 79
#$ stdout: 1225
#$ stdout: item-7
#$ assert-c: contains("stack_vec(_sb")

## ADR-141 — a list or string a function makes, fills and drops itself starts
## in a buffer of the function's own frame and moves to the heap when it
## outgrows it. Every list here must behave exactly as a heap list does:
## short ones stay in the buffer, long ones move out of it part-way through,
## and a loop's list may do either from one turn to the next.

struct Rgb:
    r: u8
    g: u8
    b: u8

fn lookups() -> int:
    m: Map[String, int] = {}
    for i in 0..200:
        m.insert(f"key{i}", i)
    total = 0
    for i in 0..200:
        total = total + m[f"key{i}"]
    return total

fn grows_past_the_buffer():
    s = String()
    for _ in 0..4:
        for c in "abcdefghijklmnopqrstuvwxy".chars():
            s.push(c)
    println(s.len(), s)

fn numbers():
    xs: Array[int] = []
    for i in 0..20:
        xs.push(i)
    xs[0] = 5
    total = 0
    for i in 0..len(xs):
        total = total + xs[i]
    println(len(xs), total, xs[0])

fn colours():
    cs: Array[Rgb] = []
    for i in 0..30:
        cs.push(Rgb(i as u8, 1, 2))
    println(len(cs), cs[29].r as int + cs[0].g as int + cs[0].b as int)

fn long_format(a: int, b: int, c: int, d: int) -> int:
    t = f"{a} and {b} and {c} and {d} and then some more words to pass sixty-four"
    return t.len()

fn sometimes_long() -> int:
    total = 0
    for n in [3, 70, 6]:
        s = String()
        for _ in 0..n:
            s.push('x')
        total = total + s.len()
    return total

fn count_down(n: int) -> int:
    label = f"level {n}"
    if n == 0:
        return label.len() - 7
    return n + count_down(n - 1)

fn make_label(n: int) -> String:
    s = f"item-{n}"
    return s

fn main():
    println(200, lookups())
    grows_past_the_buffer()
    numbers()
    colours()
    println(long_format(1000000000, 2000000000, 3000000000, 4000000000))
    println(sometimes_long())
    println(count_down(49))
    println(make_label(7))

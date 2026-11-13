# Clause BRW-15.1: "A change through one is `E3023`, with the line to write" (the lines compile,
# `[DIA-25]`), and "not a value of its own that holds views, such as an iterator a call made, which
# the name owns and may advance, `[CTL-1]`"
#$ test: run-pass
#$ rules: BRW-15, DIA-25, CTL-1
#$ stdout: 2
#$ stdout: 1
#$ stdout: 1
#$ stdout: true
#$ stdout: true

enum State:
    Walking(path: Array[int])
    Idle

struct Npc:
    state: State

struct Clock:
    frame: int

fn walk():
    n = Npc(state=State.Walking([1, 2]))
    match n.state:
        State.Walking(ref mut path):
            _ = path.pop()
            println(path.len())
        State.Idle:
            pass

fn tick(ref mut s: Clock):
    s.frame += 1

fn blend(ref mut dst: MutSpan[float], src: Span[float]):
    for i in 0..dst.len():
        dst[i] += src[i]

fn main():
    xs = [1, 2]
    it = xs.iter()
    _ = it.next()
    if it.next() exists as x:
        println(x)
    walk()
    clock = Clock(frame=0)
    tick(ref mut clock)
    println(clock.frame)
    a = [1.0, 2.0]
    b = [0.5, 0.5]
    d = ref mut a.as_mut_span()
    s = b.as_span()
    blend(ref mut d, s)
    println(a[0] == 1.5)
    tracks: Map[String, Array[float]] = {String("a"): [1.0]}
    dst_name = String("a")
    e = ref mut tracks[dst_name].as_mut_span()
    for i in 0..e.len():
        e[i] = 0.0
    println(tracks[dst_name][0] == 0.0)

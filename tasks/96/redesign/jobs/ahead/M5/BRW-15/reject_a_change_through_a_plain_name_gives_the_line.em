# Clause BRW-15.1: "A plain parameter, a plain loop, pattern or opened name (...), and a plain local
# that holds a view (...) never change what they name or what it leads to (a `Cell` field, an object
# behind a `shares` link, the elements of a `MutSpan`). A change through one is `E3023`, with the line
# to write: [four messages]"
#$ test: compile-fail
#$ rules: BRW-15, GRM-13, FN-1a, SPN-3
# The spec's four messages, each with a program whose fix compiles (`[DIA-25]`; see
# accept_the_lines_the_messages_give). `tock`, called twice, is Q12's ruling (INDEX.md): the noun is
# the type's name in lower case, and where the fix changes several calls each is named with its line
# (`[DIA-25]`, verdict 134).

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
        State.Walking(path):
            path.pop()    #$ error[E3023]: `path.pop()` changes the list in `n.state`, but `path` only reads it; to change that list through `path`, write `State.Walking(ref mut path):`
        State.Idle:
            pass

fn tick(s: Clock):
    s.frame += 1    #$ error[E3023]: `tick` sets `s.frame`, but `s` only reads the caller's clock; write `fn tick(ref mut s: Clock):` and call `tick(ref mut clock)`

fn tock(s: Clock):
    s.frame += 2    #$ error[E3023]: `tock` sets `s.frame`, but `s` only reads the caller's clock; write `fn tock(ref mut s: Clock):` and call `tock(ref mut clock)` on line 51 and `tock(ref mut spare)` on line 53

fn blend(dst: MutSpan[float], src: Span[float]):
    for i in 0..dst.len():
        dst[i] += src[i]    #$ error[E3023]: `dst[i] += src[i]` writes an element of the caller's span, but `dst` only reads it; write `fn blend(ref mut dst: MutSpan[float], src: Span[float]):` and call it `blend(ref mut d, s)`

fn zero_tracks():
    tracks: Map[String, Array[float]] = {String("a"): [1.0]}
    dst_name = String("a")
    d = tracks[dst_name].as_mut_span()
    for i in 0..d.len():
        d[i] = 0.0    #$ error[E3023]: `d[i] = 0.0` writes an element of `tracks[dst_name]`, but `d` only reads it; write `d = ref mut tracks[dst_name].as_mut_span()`

fn main():
    walk()
    clock = Clock(frame=0)
    tick(clock)
    tock(clock)
    spare = Clock(frame=0)
    tock(spare)
    xs = [1.0, 2.0]
    ys = [0.5, 0.5]
    d = ref mut xs.as_mut_span()
    s = ys.as_span()
    blend(d, s)
    zero_tracks()

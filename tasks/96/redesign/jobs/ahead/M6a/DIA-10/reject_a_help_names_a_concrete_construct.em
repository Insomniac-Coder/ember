# Clause DIA-10.1: "The help of shapes B1, B2, B4, B5 and B9 names a concrete API or construct
# (`split_at`, `retain`, `for ref mut x in xs:`, `weakref`, a capture word such as `owns n`),
# written as the line to write, never a category such as "consider restructuring"."
#$ test: compile-fail
#$ rules: DIA-10, DIA-7, BRW-18, BRW-15, CLO-4
#$ not-help: restructur
# B2, B4 and B9 with the spec's own messages, each giving the line to write; B1's concrete line is
# in DIA-7's file, and B5 has no spec text. This file's line numbers in place of the examples'.

enum State:
    Idle
    Walking(Array[int])

struct Walker:
    state: State

fn step(ref mut n: Walker):
    match n.state:
        State.Walking(path):
            path.pop()    #$ error[E3023]: `path.pop()` changes the list in `n.state`, but `path` only reads it; to change that list through `path`, write `State.Walking(ref mut path):`
        State.Idle:
            pass

fn make_counter() -> fn():
    n = 0
    return fn(): n += 1    #$ error[E3026]: the returned function changes `n`, which ends when `make_counter` returns; let the function own it: write `return fn() owns n: n += 1` in place of this line

fn main():
    counts: Map[String, int] = Map()
    counts.insert(owns String("a"), owns 1)
    for k in counts:
        counts[k] *= 2    #$ error[E3020]: `counts[k] *= 2` (line 32) changes `counts`, whose keys this loop walks (line 31); change each value through the loop's own handle: write `for k, ref mut v in counts:` in place of line 31 and `v *= 2` in place of line 32
    w = Walker(State.Walking([1, 2]))
    step(ref mut w)
    c = make_counter()
    c()

#$ test: run-pass
#$ rules: LT-1, LT-1a, TYP-14
#$ stdout: 7
#$ stdout: 2
#$ stdout: 6

## The legal side of `[LT-1]`. Each of these is a returned view that elision
## can tie to an input, and the caller holds the borrow for as long as it holds
## the result.

struct Counter:
    n: i32

    ## Rule 1: a borrowing receiver takes the return on its own.
    fn peek(mut self) -> ref i32:
        return ref self.n

## `[LT-1a]` — the attribute overrides rule 3, so the caller may keep using
## `a` while it holds the result.
@borrows(other)
fn pick(a: ref i32, other: ref i32) -> ref i32:
    return other

## Rule 2: one view-typed parameter, so the return is tied to it with nothing
## to write down.
fn give(a: ref i32) -> ref i32:
    return a

fn main():
    c: Counter = Counter(7)
    println(c.peek())

    x: i32 = 1
    y: i32 = 2
    println(pick(ref x, ref y))

    ## `[TYP-14]` — a returned reference used where a value is wanted reads
    ## through, whatever produced it. Handling that only for a named local
    ## left this printing a pointer as if it were a string.
    n: i32 = 5
    println(give(ref n) + 1)

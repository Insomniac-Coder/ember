#$ test: compile-fail
#$ rules: LT-1, LT-1a, LT-5, DIA-7
#$ error[E3062]: the returned view points into `b`, which `@borrows` does not name
#$ error[E3062]: the returned view points into `other` rather than into `self`
#$ error[E3060]: `self.n` does not live long enough

## `[LT-1a]` — `@borrows` was checked as a *signature*: that it names
## parameters, that they are view-typed, that the return is a view. The body
## was free to contradict it, and a function that promised to borrow `a` while
## returning `b` compiled. The caller then went on using `b` while holding a
## reference into it, which is the whole thing the attribute exists to prevent.

@borrows(a)
fn pick(a: ref i32, b: ref i32) -> ref i32:
    return b

struct Counter:
    n: i32

    ## `[LT-1]` rule 1 — a borrowing receiver takes the return on its own, and
    ## `[LT-1a]` says a method whose result points into an argument instead
    ## MUST say so with `@borrows`.
    fn pick(mut self, other: ref i32) -> ref i32:
        return other

    ## A by-value receiver is a copy that dies with the frame, so a reference
    ## into it dangles — the same as a borrow of a local, and the same code.
    fn peek(self) -> ref i32:
        return ref self.n

fn main():
    x: i32 = 1
    y: i32 = 2
    println(pick(ref x, ref y))
    c: Counter = Counter(3)
    println(c.pick(ref x))
    println(c.peek())

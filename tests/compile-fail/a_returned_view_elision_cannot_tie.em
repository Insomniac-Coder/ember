#$ test: compile-fail
#$ rules: LT-1, LT-1a, LT-5, DIA-7
#$ error[E3062]: `pick` returns a view of `b`, but its first line says `from a`
#$ error[E3060]: `this.n` does not live long enough

## `[LT-1a]` — `from` was checked as a *signature*: that it names a parameter,
## that it is a source, that the return is a view. The body was free to
## contradict it, and a function that promised to point into `a` while
## returning `b` compiled. The caller then went on using `b` while holding a
## reference into it, which is the whole thing `from` exists to prevent.

fn pick(a: ref i32, b: ref i32) -> ref i32 from a:
    return b

struct Counter:
    n: i32

    ## `[LT-1]` rule 1 — a method's result points into `this`; one whose body
    ## returns a view of another source parameter points, for its callers, into
    ## every source parameter (rule 3, `[LT-22]`).
    fn pick(other: ref i32) -> ref i32:
        return other

    ## An object taken over (`owns this`) is the callee's own and dies with the
    ## frame, so a reference into it dangles — the same as a borrow of a local,
    ## and the same code. (An object not taken over is the caller's: ODR-024.)
    fn peek() owns this -> ref i32:
        return ref this.n

fn main():
    x: i32 = 1
    y: i32 = 2
    println(pick(ref x, ref y))
    c: Counter = Counter(3)
    println(c.pick(ref x))
    println(c.peek())

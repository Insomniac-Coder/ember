#$ test: compile-fail
#$ rules: R12
#$ help: return fn() with owns t:
## Verdict 80, R12 (rules.md lines 749-755): a function that outlives the function that made it takes what
## it keeps with `with owns`; without a word it only borrows `t`, which ends when `make_counter` returns
## (E3026). rules.md section 3's line 1044 second message, in this program's names (the first line is
## already `-> fn()`). The line also changes more than `fn()` allows (E2113), but E3026's fix compiles
## and E2113's would not (R18, verdict 73). `Tally` has no `Copy`, so `with owns t` hands it over.

struct Tally:
    n: int = 0

fn make_counter() -> fn():
    t = Tally()
    return fn() changes t:    #$ error[E3026]: the returned function changes `t`, which ends when `make_counter` returns; let the function own it: write `return fn() with owns t:` in place of this line
        t.n += 1
        println(t.n)

fn main():
    next = make_counter()
    next()
    next()

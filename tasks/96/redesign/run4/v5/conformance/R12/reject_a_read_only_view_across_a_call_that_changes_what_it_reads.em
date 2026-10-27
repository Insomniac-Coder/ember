#$ test: compile-fail
#$ rules: R12, R8
#$ help: move line 18, `println(r)`, above line 17
## R12 (rules.md lines 717-720): across a call through a function value, "no read-only view of what they
## change ... lives across the call"; R8 (g) (lines 521-525) refuses the change at the call (E3021).
## `Counter` owns no memory, so the change is not a reshape. rules.md section 3's line 1043 message shape,
## with the one way out that applies to a change a call makes (verdict 73). The nearest allowed program is
## accept_views_used_before_the_call_that_changes_their_place.

struct Counter:
    n: int = 0

fn main():
    c = Counter()
    bump = fn(): c.n += 1
    r = ref c.n
    bump()    #$ error[E3021]: `bump()` (line 17) adds 1 to `c.n` (it runs the function made on line 15), which `r` reads (`r = ref c.n`, line 16), and `r` is used on line 18; call `bump()` after `r`'s last use: move line 18, `println(r)`, above line 17
    println(r)

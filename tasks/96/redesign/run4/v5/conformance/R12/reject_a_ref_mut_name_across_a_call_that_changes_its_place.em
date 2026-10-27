#$ test: compile-fail
#$ rules: R12, R8
#$ help: move line 17, `w += 10`, above line 16
## R12 (rules.md lines 717-720): across a call through a function value, no `ref mut` name of a place it
## changes lives; R8 (f) (lines 505-509) refuses the change through another name (E3022). rules.md
## section 3's line 1063 message shape. The nearest allowed program is
## accept_views_used_before_the_call_that_changes_their_place.

struct Counter:
    n: int = 0

fn main():
    c = Counter()
    bump = fn(): c.n += 1
    w = ref mut c.n
    bump()    #$ error[E3022]: `w` is the only name for `c.n` until its last use on line 17, and `bump()` (line 16) adds 1 to `c.n` (it runs the function made on line 14); call `bump()` after `w`'s last use: move line 17, `w += 10`, above line 16
    w += 10
    println(c.n)

#$ test: compile-fail
#$ rules: CTL-11, GRM-44
## `[CTL-11]` - the name a header declares is in scope in that block only, never in an `elif` or `else`,
## nor after the statement.

fn main():
    cur: Option[int] = Some(3)
    other: Option[int] = None
    if other exists as p:
        println(p)
    else:
        println(p)    #$ error[E1010]: cannot find `p` in this scope
    while cur exists as c:
        cur = None
    println(c)    #$ error[E1010]: cannot find `c` in this scope

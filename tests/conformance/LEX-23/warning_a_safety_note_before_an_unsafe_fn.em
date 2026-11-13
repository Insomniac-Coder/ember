#$ test: compile-pass
#$ rules: LEX-23, UNS-8, UNS-7
## `[LEX-23]` — an `unsafe fn` carries no safety note: what its callers must make true is its `@safety`
## (`[UNS-7]`), and a note before it, or at the end of its line, is warning `W3013`, whose message says to
## remove it; the note goes above each `unsafe:` block inside the function.

@safety("callers pass a valid x")
# SAFETY(aliasing): this note is on the wrong construct    #$ warning[W3013]: remove this line
unsafe fn before(x: int) -> int:
    return x

@safety("callers pass a valid y")
unsafe fn beside(y: int) -> int:  # SAFETY(intrinsic): at the end of the line    #$ warning[W3013]: remove this note
    return y

fn main():
    # SAFETY(performance): the argument is in range
    unsafe:
        println(before(1))
    unsafe:  # SAFETY(uninit): the argument is in range
        println(beside(2))

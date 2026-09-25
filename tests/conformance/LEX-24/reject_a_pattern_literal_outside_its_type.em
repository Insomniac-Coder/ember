#$ test: compile-fail
#$ rules: LEX-24, LEX-16
# D-310 — a literal pattern takes the scrutinee's type and must fit it, as any
# literal must: `-1` cannot be a `u8`, and neither can `300`. Before, `300`
# was accepted and could never match.

fn f(b: u8) -> int:
    match b:
        -1 => return 1    #$ error[E2010]: the literal `-1` does not fit in `u8`
        300 => return 2    #$ error[E2010]: the literal `300` does not fit in `u8`
        _ => return 0

fn main():
    println(f(1))

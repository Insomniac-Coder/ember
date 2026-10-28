#$ test: compile-fail
#$ rules: BRW-11
#$ help: old = copy of p
## design rules: R4
## R4: two spans of one list given to one call, one of them changeable, are proved apart or refused,
## numbers included (AN-11, E3021).

fn integrate(ref mut pos: MutSpan[float], vel: Span[float], dt: float):
    for i in 0..pos.len():
        pos[i] += vel[i] * dt

fn main():
    p = [0.0, 1.0]
    integrate(ref mut p.as_mut_span(), p.as_span(), 0.5)    #$ error[E3021]: `integrate` changes the elements of `p` through `pos` while `vel` shows the same elements; give `vel` a copy of its own: instead of this line write `old = copy of p` and `integrate(ref mut p.as_mut_span(), old.as_span(), 0.5)`
    println(p[0])

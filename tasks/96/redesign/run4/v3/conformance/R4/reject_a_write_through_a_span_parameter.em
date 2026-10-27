#$ test: compile-fail
#$ rules: R4, R5
#$ help: fn integrate(ref mut pos: MutSpan[float], ref mut vel: MutSpan[float], dt: float):
## R4, R5: a plain `Span` parameter reads; writing through it is refused, with the lines that make it
## a span that may be written (AN-11).

fn integrate(ref mut pos: MutSpan[float], vel: Span[float], dt: float):
    for i in 0..pos.len():
        pos[i] += vel[i] * dt
        vel[i] = 0.0    #$ error[E3023]: `vel[i] = 0.0` writes an element the caller gave as a `Span`, which only reads, and `vel` is a plain parameter; to write the caller's velocities, write `fn integrate(ref mut pos: MutSpan[float], ref mut vel: MutSpan[float], dt: float):` and call it `integrate(ref mut p.as_mut_span(), ref mut v.as_mut_span(), 0.5)`

fn main():
    p = [0.0, 1.0]
    v = [1.0, 1.0]
    integrate(ref mut p.as_mut_span(), v.as_span(), 0.5)
    println(p[0])

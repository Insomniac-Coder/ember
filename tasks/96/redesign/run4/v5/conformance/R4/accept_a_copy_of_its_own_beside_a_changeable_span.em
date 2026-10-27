#$ test: run-pass
#$ rules: FN-1
#$ stdout: 0.5 1.5
## design rules: R4
## R4: the nearest allowed program: `old` is a list of its own, so the two arguments are two lists and
## the call needs no check (AN-11).

fn integrate(ref mut pos: MutSpan[float], vel: Span[float], dt: float):
    for i in 0..pos.len():
        pos[i] += vel[i] * dt

fn main():
    p = [1.0, 1.0]
    old = copy of p
    old[0] = -1.0
    integrate(ref mut p.as_mut_span(), old.as_span(), 0.5)
    println(p[0], p[1])

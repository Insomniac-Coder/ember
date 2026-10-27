#$ test: run-pass
#$ rules: R5
#$ stdout: 2.0
#$ stdout: 2.0
#$ stdout: 5.0
## R5, rules section 2: `@cached` keeps the result after the first call (one flag test per call) and
## every change to a field its `reads` names clears it (one store), so the third call computes again
## (AN-21's NEW; completeness G1).

struct Mesh:
    verts: Array[f64]

    @cached
    fn radius() reads verts -> f64:
        r = 0.0
        for v in this.verts:
            r = max(r, abs(v))
        return r

fn main():
    m = Mesh(verts=[1.0, -2.0])
    println(m.radius())
    println(m.radius())
    m.verts.push(-5.0)
    println(m.radius())

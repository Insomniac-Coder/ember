#$ test: compile-fail
#$ rules: CLS-12
#$ help: remove `@cached` (line 13)
## design rules: R5, R9
## R5: a cached method may not read a field its object shares with other owners: they may change it
## without the object seeing it, so a kept result could be stale (AN-21, E2117; completeness G1).

class Shape:
    verts: Array[f64] = []

struct Model:
    shape: shares Shape

    @cached
    fn radius() -> f64:    #$ error[E2117]: `radius` reads `shape`, which its other owners may change without `Model` seeing it, so a kept result could be stale; remove `@cached` (line 13)
        r = 0.0
        for v in this.shape.verts:
            r = max(r, abs(v))
        return r

fn main():
    m = Model(shape=shares Shape(verts=[1.0, -2.0]))
    println(m.radius())

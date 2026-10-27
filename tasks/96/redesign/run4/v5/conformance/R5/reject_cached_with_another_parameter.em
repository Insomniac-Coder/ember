#$ test: compile-fail
#$ rules: R5
#$ help: radii: Map[f64, f64]
## R5: `@cached` keeps one result, so a method with another parameter is refused (AN-21, E2117;
## completeness G1).

struct Mesh:
    verts: Array[f64]

    @cached
    fn radius(scale: f64) -> f64:    #$ error[E2117]: `@cached` keeps one result, and `radius` takes `scale`; remove `@cached`, or keep the results in a field: `radii: Map[f64, f64]`
        r = 0.0
        for v in this.verts:
            r = max(r, abs(v) * scale)
        return r

fn main():
    m = Mesh(verts=[1.0, -2.0])
    println(m.radius(1.0))

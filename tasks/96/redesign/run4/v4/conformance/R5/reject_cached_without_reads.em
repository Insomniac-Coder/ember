#$ test: compile-fail
#$ rules: R5
#$ help: fn radius() reads verts -> f64:
## R5: a `@cached` method says what it reads, so a change to it can clear the kept result (AN-21,
## E2117; completeness G1).

struct Mesh:
    verts: Array[f64]

    @cached
    fn radius() -> f64:    #$ error[E2117]: say what `radius` depends on, so a change can clear it: write `fn radius() reads verts -> f64:` under `@cached`
        r = 0.0
        for v in this.verts:
            r = max(r, abs(v))
        return r

fn main():
    m = Mesh(verts=[1.0, -2.0])
    println(m.radius())

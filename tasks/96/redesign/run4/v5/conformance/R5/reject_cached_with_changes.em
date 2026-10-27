#$ test: compile-fail
#$ rules: R5
#$ help: remove `@cached` (line 12), or remove line 17
## R5: a cached method only reads: a kept result of a method that changes its object would skip the
## change on every later call (AN-21, E2117; completeness G1). What it changes is worked out from its body
## (verdict 86).

struct Mesh:
    verts: Array[f64]
    calls: int = 0

    @cached
    fn radius() -> f64:    #$ error[E2117]: a cached method only reads, and `radius` changes `calls` (line 17); remove `@cached` (line 12), or remove line 17
        r = 0.0
        for v in this.verts:
            r = max(r, abs(v))
        this.calls += 1
        return r

fn main():
    m = Mesh(verts=[1.0, -2.0])
    println(m.radius())

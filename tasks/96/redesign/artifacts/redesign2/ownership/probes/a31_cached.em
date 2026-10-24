# A31: Python's functools.cached_property spelling, on a struct.
struct Mesh:
    verts: Array[f32]

    @cached
    fn radius(self) -> f32:
        r: f32 = 0.0
        for v in self.verts:
            r = max(r, abs(v))
        return r

fn show(m: Mesh):
    println(m.radius())
    println(m.radius())

fn main():
    m = Mesh(verts=[1.0, -2.0])
    show(m)

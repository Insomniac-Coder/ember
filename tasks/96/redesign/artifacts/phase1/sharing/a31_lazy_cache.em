# A31: a read-only getter that fills a cache the first time (Python: self._radius = ...).

struct Mesh:
    verts: Array[f32]
    radius_cache: Option[f32]

    fn radius(self) -> f32:
        if self.radius_cache.is_none():
            r: f32 = 0.0
            for v in self.verts:
                r = max(r, abs(v))
            self.radius_cache = Some(r)
        return self.radius_cache.unwrap()

class MeshC:
    verts: Array[f32] = []
    radius_cache: Option[f32] = None

    fn radius(self) -> f32:
        if self.radius_cache.is_none():
            r: f32 = 0.0
            for v in self.verts:
                r = max(r, abs(v))
            self.radius_cache = Some(r)
        return self.radius_cache.unwrap()

fn main():
    m = Mesh(verts=[1.0, -2.0], radius_cache=None)
    println(m.radius())
    c = MeshC()
    println(c.radius())

// A31: a read-only getter that fills a cache the first time.
pub struct Mesh { pub verts: Vec<f32>, radius: Option<f32> }
impl Mesh {
    pub fn radius(&self) -> f32 {
        if self.radius.is_none() {
            self.radius = Some(self.verts.iter().fold(0.0f32, |m, v| m.max(v.abs())));
        }
        self.radius.unwrap()
    }
}

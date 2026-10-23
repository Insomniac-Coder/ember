// A10: two entities share one mesh; then one of them edits it.
use std::rc::Rc;
pub struct Mesh { pub verts: Vec<f32> }
pub struct Entity { pub mesh: Box<Mesh> }
pub struct Entity2 { pub mesh: Rc<Mesh> }

pub fn build() -> (Entity, Entity) {
    let m = Box::new(Mesh { verts: vec![0.0; 9] });
    let a = Entity { mesh: m };
    let b = Entity { mesh: m };
    (a, b)
}

// Switch to Rc to share it: now nobody may change it.
pub fn edit(a: &mut Entity2) {
    a.mesh.verts.push(1.0);
}

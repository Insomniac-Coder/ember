// A3: two mutable references into one container.
pub struct Body { pub pos: f32, pub vel: f32 }

fn collide(a: &mut Body, b: &mut Body) {
    std::mem::swap(&mut a.vel, &mut b.vel);
}
pub fn step(bodies: &mut Vec<Body>, i: usize, j: usize) {
    collide(&mut bodies[i], &mut bodies[j]);
}

// A3 workarounds: get_disjoint_mut (stable since 1.86) or split_at_mut.
pub struct Body { pub pos: f32, pub vel: f32 }

fn collide(a: &mut Body, b: &mut Body) {
    std::mem::swap(&mut a.vel, &mut b.vel);
}
pub fn step(bodies: &mut Vec<Body>, i: usize, j: usize) {
    if let Ok([a, b]) = bodies.get_disjoint_mut([i, j]) {
        collide(a, b);
    }
}
pub fn step_split(bodies: &mut Vec<Body>, i: usize, j: usize) {
    assert!(i < j);
    let (left, right) = bodies.split_at_mut(j);
    collide(&mut left[i], &mut right[0]);
}

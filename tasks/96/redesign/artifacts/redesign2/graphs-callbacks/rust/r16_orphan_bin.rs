// A16: is the orphan rule applied to a program that nobody can depend on (a binary crate)?
struct Vec3 { x: f32 }
impl std::ops::Neg for Vec<Vec3> {
    type Output = Vec<Vec3>;
    fn neg(self) -> Vec<Vec3> { self.into_iter().map(|v| Vec3 { x: -v.x }).collect() }
}
fn main() {
    let v = vec![Vec3 { x: 1.0 }];
    println!("{}", (-v)[0].x);
}

// A2 workaround: copy what you need out, push, then index again.
pub struct Enemy { pub x: f32, pub hp: i32 }

pub fn split_boss(enemies: &mut Vec<Enemy>) {
    let (x, hp) = (enemies[0].x, enemies[0].hp);
    enemies.push(Enemy { x: x + 1.0, hp: hp / 2 });
    enemies[0].hp /= 2;
}

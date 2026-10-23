// A2: add to a container while a reference into it is in use.
pub struct Enemy { pub x: f32, pub hp: i32 }

pub fn split_boss(enemies: &mut Vec<Enemy>) {
    let boss = &mut enemies[0];
    enemies.push(Enemy { x: boss.x + 1.0, hp: boss.hp / 2 });
    boss.hp /= 2;
}

// A4 workaround: collect spawns in a second list, then retain + extend.
pub struct Enemy { pub hp: i32, pub splits: bool }

pub fn update(enemies: &mut Vec<Enemy>) {
    let mut spawned = Vec::new();
    for e in enemies.iter() {
        if e.hp <= 0 && e.splits {
            spawned.push(Enemy { hp: 5, splits: false });
        }
    }
    enemies.retain(|e| e.hp > 0);
    enemies.extend(spawned);
}

// A4: spawn and despawn while iterating the list.
pub struct Enemy { pub hp: i32, pub splits: bool }

pub fn update(enemies: &mut Vec<Enemy>) {
    for (i, e) in enemies.iter().enumerate() {
        if e.hp <= 0 {
            if e.splits {
                enemies.push(Enemy { hp: 5, splits: false });
            }
            enemies.remove(i);
        }
    }
}

// Rust twin of the copy loop: for e in enemies[:] while changing enemies.
pub struct Enemy { pub hp: i32 }
pub fn update(enemies: &mut Vec<Enemy>) {
    for e in enemies[..].iter_mut() {
        e.hp -= 1;
        if e.hp <= 0 { enemies.push(Enemy { hp: 9 }); }
    }
}

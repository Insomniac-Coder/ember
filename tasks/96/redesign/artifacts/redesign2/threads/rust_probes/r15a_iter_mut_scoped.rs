// A15: each scoped thread changes its own element (Rust accepts disjoint &mut through iter_mut).
pub struct Player { pub hp: i32 }
pub fn hurt_all(players: &mut Vec<Player>) {
    std::thread::scope(|s| {
        for p in players.iter_mut() {
            s.spawn(move || p.hp -= 1);
        }
    });
}

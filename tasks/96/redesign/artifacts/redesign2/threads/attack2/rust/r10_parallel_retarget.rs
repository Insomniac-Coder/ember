// Twin of y22: each thread re-targets its own players; the target is an Arc (atomic increment for the
// new target, atomic decrement for the old one).
use std::sync::Arc;
use std::thread;
pub struct Enemy { pub x: f64 }
pub struct Player { pub x: f64, pub target: Option<Arc<Enemy>> }
pub fn aim(players: &mut [Player], enemies: &[Arc<Enemy>]) {
    thread::scope(|s| {
        for part in players.chunks_mut(1024) {
            s.spawn(move || {
                for p in part.iter_mut() {
                    let mut best = &enemies[0];
                    for e in enemies { if (e.x - p.x).abs() < (best.x - p.x).abs() { best = e; } }
                    p.target = Some(best.clone());
                }
            });
        }
    });
}

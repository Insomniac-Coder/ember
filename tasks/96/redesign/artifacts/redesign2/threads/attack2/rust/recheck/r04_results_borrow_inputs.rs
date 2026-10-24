// Rust: scoped threads build result objects that point at the input objects (a lifetime on Match).
struct Player { name: String }
struct Match<'a> { player: &'a Player, score: usize }
fn main() {
    let players: Vec<Player> = (0..4).map(|i| Player { name: format!("p{i}") }).collect();
    let matches: Vec<Match> = std::thread::scope(|s| {
        let hs: Vec<_> = players.iter().map(|p| s.spawn(move || Match { player: p, score: p.name.len() })).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for m in &matches { println!("{} {}", m.player.name, m.score); }
}

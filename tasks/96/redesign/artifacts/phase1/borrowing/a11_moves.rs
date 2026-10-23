// A11: use after move, partial moves, moves in a loop.
pub struct Msg { pub to: String, pub body: String }
fn send(_m: Msg) {}
fn log(_m: &Msg) {}

pub fn notify(m: Msg) {
    send(m);
    log(&m);
}

pub struct Player { pub name: String, pub inventory: Vec<String> }
fn keep(_p: Player) {}

pub fn rename(p: Player) {
    let n = p.name;
    println!("{}", n);
    keep(p);
}

pub fn spawn_all(template: Player, n: usize) -> Vec<Player> {
    let mut v = Vec::new();
    for _ in 0..n {
        v.push(template);
    }
    v
}

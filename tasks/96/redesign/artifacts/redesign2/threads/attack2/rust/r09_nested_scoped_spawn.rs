// Twin of x08: a scoped task spawns another task into the same scope. Rust's Scope is Sync.
use std::thread;
pub fn go() {
    let xs = vec![1, 2, 3];
    thread::scope(|s| {
        s.spawn(|| { s.spawn(|| println!("{}", xs[0])); });
        println!("{}", xs[1]);
    });
}

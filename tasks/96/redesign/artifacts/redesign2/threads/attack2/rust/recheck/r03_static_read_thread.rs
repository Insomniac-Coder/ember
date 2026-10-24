// Rust: a worker thread reads a module-level constant.
static DEBUG: bool = true;
fn main() {
    let h = std::thread::spawn(|| DEBUG);
    println!("{}", h.join().unwrap());
}

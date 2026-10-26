use std::sync::Mutex;
use std::thread;

static RESULTS: Mutex<Vec<i32>> = Mutex::new(Vec::new());

fn main() {
    RESULTS.lock().unwrap().push(1);
    let g = RESULTS.lock().unwrap();
    for &r in g.iter() {
        let t = thread::spawn(move || {
            RESULTS.lock().unwrap().push(r * r);
        });
        t.join().unwrap();
    }
}

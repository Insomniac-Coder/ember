use std::sync::Mutex;
use std::thread;

static RESULTS: Mutex<Vec<i32>> = Mutex::new(Vec::new());

fn main() {
    let g = RESULTS.lock().unwrap();
    for &r in g.iter() {
        println!("{r}");
        thread::spawn(move || {
            RESULTS.lock().unwrap().push(r * r);
        });
    }
}

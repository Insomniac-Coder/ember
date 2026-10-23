// A33: a shared score table behind a lock; hand out the top name.
use std::sync::Mutex;
pub fn top(scores: &Mutex<Vec<String>>) -> &String {
    &scores.lock().unwrap()[0]
}

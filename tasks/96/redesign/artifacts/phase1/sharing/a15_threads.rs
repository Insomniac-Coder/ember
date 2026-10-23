// A15: count words in several texts on several threads.
use std::rc::Rc;
use std::thread;

pub fn count_words(texts: &Vec<String>) -> usize {
    let mut total = 0;
    let mut handles = Vec::new();
    for t in texts {
        handles.push(thread::spawn(|| { total += t.split_whitespace().count(); }));
    }
    for h in handles { h.join().unwrap(); }
    total
}

pub fn share_rc() {
    let names = Rc::new(vec![String::from("a")]);
    let h = thread::spawn(move || names.len());
    h.join().unwrap();
}

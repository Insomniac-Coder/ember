// A14: a server task that keeps a cache and awaits; the executor needs Send + 'static futures.
use std::future::Future;
use std::rc::Rc;

pub fn spawn<F: Future<Output = ()> + Send + 'static>(_f: F) {}
async fn fetch() -> usize { 1 }

pub fn serve_rc() {
    let cache = Rc::new(vec![1, 2, 3]);
    spawn(async move {
        let n = fetch().await;
        println!("{}", cache.len() + n);
    });
}

pub fn serve_borrow() {
    let name = String::from("player1");
    spawn(async {
        let n = fetch().await;
        println!("{} {}", name, n);
    });
}

// Polling a stored future needs Pin.
pub fn poll_all(tasks: &mut Vec<Box<dyn Future<Output = ()>>>, cx: &mut std::task::Context<'_>) {
    for t in tasks.iter_mut() {
        let _ = t.poll(cx);
    }
}

// sizes.rs: the heap block each Rust program allocates per player (RcBox = two counts + the value).
use std::cell::{Cell, RefCell};
#[allow(dead_code)] struct Player { hp: i32, x: i32, inv: Vec<i32> }
#[allow(dead_code)] struct PlayerC { hp: Cell<i32>, x: i32, inv: RefCell<Vec<i32>> }
fn main() {
    println!("refcell payload {} + 16 count bytes = {}", std::mem::size_of::<RefCell<Player>>(), std::mem::size_of::<RefCell<Player>>() + 16);
    println!("cell payload {} + 16 count bytes = {}", std::mem::size_of::<PlayerC>(), std::mem::size_of::<PlayerC>() + 16);
}

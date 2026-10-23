// A9 workaround: Rc<RefCell<T>>. Compiles; the borrow inside the loop panics at run time.
use std::cell::RefCell;
use std::rc::Rc;
pub struct Player { pub hp: i32, pub inventory: Vec<String> }
pub struct Camera { pub target: Rc<RefCell<Player>> }
pub struct Hud { pub target: Rc<RefCell<Player>> }

fn main() {
    let p = Rc::new(RefCell::new(Player { hp: 100, inventory: Vec::new() }));
    let cam = Camera { target: Rc::clone(&p) };
    let hud = Hud { target: Rc::clone(&p) };
    cam.target.borrow_mut().hp -= 1;
    hud.target.borrow_mut().inventory.push("key".to_string());
    // Walk the inventory while the camera reacts: a run-time borrow panic.
    for item in hud.target.borrow().inventory.iter() {
        cam.target.borrow_mut().hp -= item.len() as i32;
    }
}

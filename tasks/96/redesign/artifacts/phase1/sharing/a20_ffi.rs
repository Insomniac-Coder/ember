// A20: a C event library calls back into the game.
use std::os::raw::c_void;
extern "C" { fn register_cb(cb: extern "C" fn(*mut c_void), ud: *mut c_void); }
pub struct Game { pub score: i32 }

pub fn setup(game: &mut Game) {
    let cb = |_ud: *mut c_void| game.score += 1;
    unsafe { register_cb(cb, std::ptr::null_mut()) }
}

// The workaround: a raw pointer as user data, turned back into a reference of any lifetime.
extern "C" fn on_event(ud: *mut c_void) {
    let g: &'static mut Game = unsafe { &mut *(ud as *mut Game) };
    g.score += 1;
}
pub fn setup_ok(game: &mut Game) {
    unsafe { register_cb(on_event, game as *mut Game as *mut c_void) }
}

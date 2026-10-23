use std::os::raw::c_void; extern "C" { fn register_cb(cb: extern "C" fn(*mut c_void), ud: *mut c_void); } pub struct Game { pub score: i32 }
// The workaround: a raw pointer as user data, turned back into a reference of any lifetime.
extern "C" fn on_event(ud: *mut c_void) {
    let g: &'static mut Game = unsafe { &mut *(ud as *mut Game) };
    g.score += 1;
}
pub fn setup_ok(game: &mut Game) {
    unsafe { register_cb(on_event, game as *mut Game as *mut c_void) }
}

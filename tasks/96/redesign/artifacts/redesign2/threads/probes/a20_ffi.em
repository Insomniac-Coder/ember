# A20: a C event library calls back into the game.

class Game:
    score: i32 = 0

@sync
class SyncGame:
    score: i32 = 0

from std.ffi import ForeignCallback, Retained

unsafe extern "C":
    fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    fn register_raw(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)

# Without a contract: a capturing lambda where C wants a function pointer.
fn setup_raw(game: Game):
    # SAFETY(ffi): test of the refusal
    unsafe:
        register_raw(fn(ud: *mut void): game.score += 1, null[*mut void]())

# With the contract: the trampoline and the user data are generated; no unsafe.
fn setup(game: Game):
    token = register_cb(owned fn(): game.score += 1)

fn main():
    g = Game()
    setup_raw(g)
    setup(g)

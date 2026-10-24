# A20 proposed (a): a C function taking a callback and its user data, declared with no contract.
class Game:
    score: i32 = 0

unsafe extern "C":
    fn unregister_cb(cb: extern "C" fn(*mut void), ud: *mut void)
    fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)

fn setup(game: Game):
    token = register_cb(fn(): game.score += 1)

fn main():
    setup(Game())

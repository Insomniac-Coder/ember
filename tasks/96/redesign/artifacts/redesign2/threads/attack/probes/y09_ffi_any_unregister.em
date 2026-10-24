# PROPOSED syntax. A threads(any) callback (C calls it from its own worker thread); the token is dropped on
# the main thread while a call is running. The generation check passed before the drop, so the closure's
# state is freed under the running call (bench/ffi_toctou.c, FIX=0: "wrote into freed state: YES").
@ffi(param(cb, callback(retained, user_data=ud), threads(any)), destroys(unregister))
extern "C" fn register_cb(cb: fn(ud: *mut void), ud: *mut void) -> CallbackToken

fn main():
    game = Game()
    token = register_cb(fn(): game.score += 1)
    thread.sleep_ms(20)
    drop(token)                     # unregister: frees the closure's state, releases `game`

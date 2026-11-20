# Clause FFI-23.1: "A token is bound to the thread that registered the callback, whatever the
# callback's contract (`[THR-8]`: neither `Send` nor `ThreadShared`; handing it to another thread is
# `E7008`"
#$ test: compile-fail
#$ rules: FFI-23, THR-8, THR-17
# The spec's message, with this file's line numbers: the callback's contract is `threads(any)`, and
# the token still stays on its thread. The declarations are never linked.

import std.thread

unsafe extern "C":  # SAFETY(ffi): the declarations this test needs; it is never run
    @safety("`cb` and `ud` are a pair `register_cb` was given")
    unsafe fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(any)), destroys(unregister))
    safe fn register_cb(owns cb: thread_shared fn(), ud: *mut void)

fn main():
    token = register_cb(owns fn(): println("hi"))
    h = thread.spawn(owns fn() owns token: mem.drop(owns token))    #$ error[E7008]: `token` keeps the callback registered on line 18 for C, and its end unregisters the callback on the thread that holds the token; keep `token` in `main`, or end it there before the thread starts: `mem.drop(owns token)` above line 19
    h.join()

#$ test: compile-fail
#$ rules: FFI-33b, FFI-8, THR-8
# `[FFI-33b]`, `[THR-8]` — an `owns T` of a type whose contract says `threads(creator)` or
# `threads(main)` is not `Send`: sent to another thread it is `E7008` (a thread-bound value), not
# `E7004`, which is any other value that may not move.

import std.thread

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(destroys(gadget_close), threads(creator))
    type Gadget
    @ffi(destroys(panel_close), threads(main))
    type Panel
    @safety("`gadget` is a gadget C made, not closed yet")
    unsafe fn gadget_close(gadget: *mut Gadget)
    @safety("`panel` is a panel C made, not closed yet")
    unsafe fn panel_close(panel: *mut Panel)
    @ffi(result(returns_owned(destructor=gadget_close), one))
    safe fn gadget_new() -> owns Gadget
    @ffi(result(returns_owned(destructor=panel_close), one))
    safe fn panel_new() -> owns Panel

fn look_gadget(gadget: owns Gadget):
    pass

fn look_panel(panel: owns Panel):
    pass

fn main():
    gadget = gadget_new()
    panel = panel_new()
    a = thread.spawn(owns owned fn() => look_gadget(gadget))    #$ error[E7008]: the thread's function captures `gadget`, an `owns Gadget`, which is not Send
    b = thread.spawn(owns owned fn() => look_panel(panel))    #$ error[E7008]: the thread's function captures `panel`, an `owns Panel`, which is not Send
    a.join()
    b.join()

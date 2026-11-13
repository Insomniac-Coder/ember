#$ test: run-fail
#$ rules: FFI-33b, COST-6
#$ profiles: debug, development, release
#$ panics: `gadget`, a `Gadget` made on thread 2 (line 43), ended on thread 1
#$ panics: ; a `threads(creator)` value ends where it was made
#$ stdout: taken
# `[FFI-33b]` — a type is bound to its thread when a `returns_owned` result giving an `owns T` of it
# says `threads(creator)`, as here. The end of such a value compares the thread that made it with the
# one it ends on, in every profile, and the stop names the local that ends (`gadget`), the type, the
# line that made it, which is the call's (line 50), not the declaration's, and both threads (`[COST-6]`'s
# table). Safe code never hands one to another thread; here a carrier the program declares `Send` by
# hand moves one made on a worker back to the main thread, as unsafe code or C may, where `main` takes it
# out into a local of its own.

import std.thread

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    type Gadget
    @ffi(link_name="named_gadget_close")
    @safety("`gadget` is a gadget C made, not closed yet")
    unsafe fn gadget_close(gadget: *mut Gadget)
    @ffi(result(returns_owned(destructor=gadget_close), one, threads(creator)), link_name="named_gadget_new")
    safe fn gadget_new() -> owns Gadget

pub extern "C" fn named_gadget_new() -> *mut Gadget:
    # SAFETY: an address no Ember value has, which only `named_gadget_close` sees
    unsafe:
        return 8192 as *mut Gadget

pub extern "C" fn named_gadget_close(gadget: *mut Gadget):
    pass

struct Carrier:
    gadget: owns Gadget

unsafe extend Carrier implements Send:  # SAFETY: wrongly: the gadget is bound to the thread that made it, which the stop catches
    pass

fn look(gadget: owns Gadget):
    pass

fn made() -> Carrier:
    return Carrier(gadget=gadget_new())

fn main():
    worker = thread.spawn(owns owned fn() -> Carrier => made())
    carrier = worker.join()
    gadget = owns carrier.gadget
    look(gadget)
    println("taken")

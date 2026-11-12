#$ test: run-fail
#$ rules: FFI-33b, FFI-36b, COST-6
#$ profiles: debug, development, release
#$ panics: a `Gadget` made on thread 1 (line 46), ended on thread
#$ panics: ; a `threads(creator)` value ends where it was made
# `[FFI-33b]` — an `owns T` of a type whose contract says `threads(creator)` keeps the thread that
# made it, and its end compares that thread with the one it ends on, in every profile, stopping
# the program on another, naming the type, the line that made it and both threads (`[COST-6]`'s
# table). Safe code never hands one to another thread (it is not `Send`), so the stop backs unsafe
# code and C: here a carrier the program declares `Send` by hand moves one to a thread, where it
# ends. The C side is the program's own C functions, so the test links on its own.

from std.ffi import adopt
import std.thread

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(destroys(gadget_close), threads(creator))
    type Gadget
    @ffi(link_name="carried_gadget_new")
    @safety("C gives a fresh gadget, which the caller owns")
    unsafe fn gadget_new() -> *mut Gadget
    @ffi(link_name="carried_gadget_close")
    @safety("`gadget` is a gadget C made, not closed yet")
    unsafe fn gadget_close(gadget: *mut Gadget)

pub extern "C" fn carried_gadget_new() -> *mut Gadget:
    # SAFETY: an address no Ember value has, which only `carried_gadget_close` sees
    unsafe:
        return 4096 as *mut Gadget

pub extern "C" fn carried_gadget_close(gadget: *mut Gadget):
    pass

struct Carrier:
    gadget: owns Gadget

unsafe extend Carrier implements Send:  # SAFETY: wrongly: the gadget is bound to the thread that made it, which the stop catches
    pass

fn look(carrier: Carrier):
    pass

fn made() -> owns Gadget:
    # SAFETY: a fresh gadget, owned only by this `owns Gadget`
    unsafe:
        return adopt(gadget_new())

fn main():
    carrier = Carrier(gadget=made())
    worker = thread.spawn(owns owned fn() => look(carrier))
    worker.join()
    println("unreachable")

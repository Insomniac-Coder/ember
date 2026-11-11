#$ test: run-pass
#$ rules: FFI-54, FFI-36b, FFI-36a
#$ profiles: debug, development, release
#$ stdout: true
#$ stdout: closed 1
# `[FFI-54]` — `into_raw()` on an `owns T` of a foreign type gives C the object's address and
# stops Ember ending it: its destructor is not called, and the address leaves the runtime's record
# of the objects Ember owns (`[FFI-36b]`), so adopting it again is no second owner. The C side is
# the program's own C functions, so the test links on its own.

from std.ffi import adopt

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(destroys(gadget_close))
    type Gadget
    @ffi(link_name="given_gadget_new")
    @safety("C gives a fresh gadget, which the caller owns")
    unsafe fn gadget_new() -> *mut Gadget
    @ffi(link_name="given_gadget_close")
    @safety("`gadget` is a gadget C made, not closed yet")
    unsafe fn gadget_close(gadget: *mut Gadget)

pub extern "C" fn given_gadget_new() -> *mut Gadget:
    # SAFETY: an address no Ember value has, which only `given_gadget_close` sees
    unsafe:
        return 4096 as *mut Gadget

pub extern "C" fn given_gadget_close(gadget: *mut Gadget):
    println("closed 1")

fn made() -> owns Gadget:
    # SAFETY: a fresh gadget, owned only by this `owns Gadget`
    unsafe:
        return adopt(gadget_new())

fn main():
    first = made()
    # SAFETY: the address is taken back below, once
    unsafe:
        address = first.into_raw()
        _again = adopt(address)
        println(address == gadget_new())

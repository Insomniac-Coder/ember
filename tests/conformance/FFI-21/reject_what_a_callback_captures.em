#$ test: compile-fail
#$ rules: FFI-21, FFI-22, FFI-33b
#$ profiles: debug
# `[FFI-21]`, `[FFI-22]` — C may call a callback from any thread, so what it captures is `Send`
# (`E7004`) unless its contract says `threads(main)` or `threads(creator)`; one C keeps past the
# call (`once`, `retained`) may outlive every local here, so it borrows nothing (`E3026`: write it
# `owned fn`).

from std.ffi import ForeignCallback

class Counter:
    count: i32 = 0

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @safety("callers pass what the C function `forget` requires")
    fn forget(handler: extern "C" fn(*mut void), data: *mut void)
    @ffi(param(handler, callback(borrowed, user_data=data)))
    safe fn now(handler: extern "C" fn(*mut void), data: *mut void)
    @ffi(param(handler, callback(borrowed, user_data=data), threads(main)))
    safe fn now_on_main(handler: extern "C" fn(*mut void), data: *mut void)
    @ffi(param(handler, callback(once, user_data=data)))
    safe fn later(handler: extern "C" fn(*mut void), data: *mut void)
    @ffi(param(handler, callback(retained, user_data=data), threads(creator)), destroys(forget))
    safe fn always(handler: extern "C" fn(*mut void), data: *mut void)

fn main():
    counter = Counter()
    name = "kept".to_string()
    now(fn(): counter.count += 1)    #$ error[E7004]: captures `counter`
    now_on_main(fn(): counter.count += 1)
    later(fn(): println(name))    #$ error[E3026]: borrows `name`, and C keeps it past the call
    later(owned fn(): println(counter.count))    #$ error[E7004]: captures `counter`, a `Counter`, which is not Send
    token = always(owned fn(): println(counter.count))
    println(name)

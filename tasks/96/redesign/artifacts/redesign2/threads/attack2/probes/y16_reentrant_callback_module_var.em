# Proposed syntax (module variables, answers report 6.3; inferred contracts, 8.4). The callback C runs
# during a later C call, on the main thread (threads(main)), so A19 rule 5 lets it touch LOG. The loop
# holds a loan into LOG's buffer across `lib.poll_events()`. If the foreign function's summary is what its
# declaration states (here `@ffi(effects=[Io])`: it does input and output), it reshapes no Ember place,
# and the callback's push reallocates LOG under the loop.
# Today's twin (probe x06, a class field instead of a module variable) passes `ember check`; what stops
# it at run time today is the field's access word ([EXC-1], [EXC-19]), which the proposed model removes.
LOG: Array[String] = []

unsafe extern "C":
    @ffi(param(cb, callback(retained, user_data=ud), threads(main)), destroys(off_key))
    safe fn on_key(cb: extern "C" fn(*mut void, i32), ud: *mut void)
    fn off_key(cb: extern "C" fn(*mut void, i32), ud: *mut void)
    @ffi(effects=[Io])
    safe fn poll_events()          # C: reads the window's events and calls the key callback

fn main():
    token = on_key(fn(k: i32): LOG.push(f"key {k}"))
    LOG.push("start")
    for line in LOG:
        poll_events()              # the callback runs here and pushes to LOG
        println(line)

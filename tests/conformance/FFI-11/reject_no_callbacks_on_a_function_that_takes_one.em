#$ test: compile-fail
#$ rules: FFI-11, FFI-52
# `[FFI-52]` — `no_callbacks` on a function whose own contract takes a callback is `E5069`: it may
# call back into Ember. So is one on a function that takes a function pointer.

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @safety("`token` is one `subscribe` gave")
    unsafe fn unsubscribe(token: i32)
    @ffi(param(hook, callback(retained, user_data=data)), destroys(unsubscribe), no_callbacks)    #$ error[E5069]: `subscribe` takes a callback (`param(hook, callback(retained, user_data=data))`), so it may call back into Ember; remove `no_callbacks` from its `@ffi(...)`
    safe fn subscribe(hook: extern "C" fn(*mut void, i32) -> i32, data: *mut void) -> i32
    @ffi(no_callbacks)    #$ error[E5069]: `sort_with` takes a function pointer (`compare`), so it may call back into Ember
    @safety("`compare` orders two of the `n` values at `values`")
    unsafe fn sort_with(values: *mut i32, n: usize, compare: extern "C" fn(*i32, *i32) -> i32)

fn main():
    pass

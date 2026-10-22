# The overlay of ../reject_cpp_callback_contracts.em: `threads(creator)` and `handle` written on a
# C++ function's entry, where neither belongs ([FFI-1], [FFI-11], ODR-247).

unsafe overlay cpp "hooks.hpp":
    fn maybe_hook(hook: callback(borrowed, user_data=data) nullable, data, value)
    @ffi(threads(creator))
    fn given_both()
    @ffi(handle)
    fn watching()
    fn new()
    fn on_resize(hook: callback(borrowed, user_data=data) nullable, data, size)

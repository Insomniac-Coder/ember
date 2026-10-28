# The overlay of ../reject_cpp_callback_contracts.em: a user data's place the callback does not have
# (ODR-245), and `threads(creator)` and `handle` written on a C++ function's entry, where neither
# belongs ([FFI-1], [FFI-11], ODR-247).

unsafe overlay cpp "hooks.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    fn pick_with(pick: callback(borrowed, user_data=right@3), right)
    fn maybe_hook(hook: callback(borrowed, user_data=data) nullable, data, value)
    @ffi(threads(creator))
    fn given_both()
    @ffi(handle)
    fn watching()
    fn new()
    fn on_resize(hook: callback(borrowed, user_data=data) nullable, data, size)

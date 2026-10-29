# The overlay of ../accept_cpp_callback_contracts.em: the callback contracts of ODR-245, ODR-246
# and ODR-253 on C++ functions and a method ([FFI-21], [FFI-11]).

unsafe overlay cpp "hooks.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    fn pick_with(pick: callback(borrowed, user_data=right@2), right)
    fn maybe_hook(hook: callback(borrowed, user_data=data) nullable, data, value)
    fn given_both()
    @ffi(destroys(cancel))
    fn watch(hook: callback(retained, user_data=data) nullable, data)
    fn poke(value)
    fn watching()
    hide cancel
    class Panel:
        fn new()
        fn on_resize(hook: callback(borrowed, user_data=data) nullable, data, size)

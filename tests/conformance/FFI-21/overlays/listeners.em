# The overlay of ../accept_a_cpp_callback_with_user_data.em: each callback's user data named, and
# each call asserted safe ([FFI-21], [FFI-2]).

unsafe overlay cpp "listeners.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    fn new()
    fn add(value)
    fn each(visit: callback(borrowed, user_data=context), context)
    fn sum_with(map: callback(borrowed, user_data=context), context, n)
    @ffi(destroys(unlisten))
    fn listen(listener: callback(retained, user_data=data), data)
    fn emit(event)
    fn listening()
    hide unlisten

# The overlay of ../accept_callback_contracts_through_an_overlay.em: a user data named by its place
# (ODR-245), callbacks C may be given none of (ODR-253), and a registration undone by the id it
# returned (ODR-246).

unsafe overlay c "registry.h":
    fn each_pair(count, visit: callback(borrowed, user_data=context@2), context)
    fn maybe_call(hook: callback(borrowed, user_data=data) nullable, data, value)
    fn given()
    @ffi(destroys(unsubscribe))
    fn subscribe(hook: callback(retained, user_data=data), data)
    fn fire(value)
    fn subscribers()
    hide unsubscribe

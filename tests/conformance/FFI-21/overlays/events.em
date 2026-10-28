# The overlay of ../accept_callbacks_c_keeps_past_the_call.em.

unsafe overlay c "events.h":  # SAFETY(ffi): each entry matches its declaration in the header
    fn run_later(value, done: callback(once, user_data=data), data)
    fn run_pending()
    @ffi(destroys(unlisten))
    fn listen(kind, listener: callback(retained, user_data=data), data)
    fn emit(kind, event)
    fn listening()
    hide unlisten

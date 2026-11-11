# The overlay of ../accept_a_callback_without_a_thread_word_is_any.em and
# ../reject_a_callback_any_thread_may_call_holds_a_one_thread_value.em: no `threads(...)` word, so
# the default, `threads(any)`.

unsafe overlay c "ticks.h":  # SAFETY(ffi): each entry matches its declaration in the header
    fn tick_here(tick: callback(borrowed, user_data=data), data)
    fn tick_elsewhere(tick: callback(borrowed, user_data=data), data)

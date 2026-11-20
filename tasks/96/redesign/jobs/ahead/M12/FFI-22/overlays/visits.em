# The overlay of ../reject_a_borrowed_callback_any_thread_may_run_changes_a_list.em: no `threads(...)`
# word, so the default, `threads(any)`.

unsafe overlay c "visits.h":  # SAFETY(ffi): each entry matches its declaration in the header
    fn visit_each(count, visit: callback(borrowed, user_data=data), data)

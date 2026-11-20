# The overlay of ../reject_ffi_safe_on_a_function_pointer_that_takes_a_pointer.em: `log_fn` takes a
# raw pointer, which no assertion covers.

unsafe overlay c "hooks.h":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(safe)
    type log_fn
    fn get_log()

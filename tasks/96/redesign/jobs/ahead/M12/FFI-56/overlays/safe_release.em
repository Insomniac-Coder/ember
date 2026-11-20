# The overlay of ../accept_a_c_function_pointer_asserted_safe.em: `release_fn` takes a scalar, and
# its entry asserts it safe to call.

unsafe overlay c "hooks.h":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(safe)
    type release_fn
    fn get_release()

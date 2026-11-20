# The overlay of ../reject_a_c_function_pointer_called_outside_unsafe.em: `get_release` is safe to
# call; the pointer it gives has no entry.

unsafe overlay c "hooks.h":  # SAFETY(ffi): each entry matches its declaration in the header
    fn get_release()

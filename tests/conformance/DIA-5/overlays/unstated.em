# The overlay of ../reject_an_overlay_error_names_the_c_declaration.em: an entry asserts a call
# safe while leaving a pointer without facts.

unsafe overlay c "labels.h":  # SAFETY(ffi): each entry matches its declaration in the header
    fn label_sum(items, n)

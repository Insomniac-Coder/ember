# The overlay of ../reject_a_binding_that_hands_c_text_to_a_strview.em.

unsafe overlay c "foreign_text.h":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(macro_fn)
    fn GREETING(n: i32) -> StrView

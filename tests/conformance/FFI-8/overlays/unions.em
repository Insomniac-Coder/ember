# The overlay of ../accept_unions_are_layout_c_unions.em.

unsafe overlay c "unions.h":  # SAFETY(ffi): each entry matches its declaration in the header
    fn number_of_float(f)
    fn number_bits(n)
    fn number_twice(n: borrowed one)
    fn value_real(d)
    fn value_half(v)

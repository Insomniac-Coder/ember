# The overlay of ../accept_long_double_is_c_longdouble.em.

unsafe overlay c "precise.h":
    @ffi(effects=[], deterministic)
    fn precise_third()
    @ffi(effects=[], deterministic)
    fn precise_twice(x)
    @ffi(effects=[], deterministic)
    fn precise_to_double(x)
    fn precise_measure(tag)

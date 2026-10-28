# Another overlay of classes.hpp (D-879): the function alone.

unsafe overlay cpp "classes.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    fn doubled(n)

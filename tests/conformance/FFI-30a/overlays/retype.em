# An overlay that tries to change a type's field.

unsafe overlay c "pair.h":  # SAFETY(ffi): each entry matches its declaration in the header
    class Pair:
        left: i64

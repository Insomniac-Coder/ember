# The overlay of ../reject_a_count_naming_no_length.em: a method's count names no parameter.

unsafe overlay cpp "texts.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    class Tally:
        fn new()
        fn add(values: borrowed count(total), count)

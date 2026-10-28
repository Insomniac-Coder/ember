# The overlay of ../reject_a_method_line_wins_over_the_body.em: `lookups` restated as `mut self`,
# though its body changes nothing.

unsafe overlay cpp "cache.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    class Cache:
        fn lookups(mut self) -> c_int

# The overlay of ../accept_views_and_optional_values_cross_safely.em: each call asserted safe, and
# each view result's lifetime ([FFI-11]).

unsafe overlay cpp "views.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    fn vowels(text)
    fn sum(values)
    fn negate(values)
    fn shout_length(text)
    fn new()
    fn first() -> from(self)
    fn scores() -> from(self)
    fn scores_mut() -> from(self)
    fn total()
    fn greeting() -> from(static)
    fn trim(text) -> from(text)
    fn digit(code)
    fn or_zero(value)
    fn scaled(factor, x)
    fn value()
    fn make_box(value)
    fn consume(box)
    fn boxes_destroyed()

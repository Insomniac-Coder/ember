# The overlay of ../accept_a_declared_macro_is_called_through_its_wrapper.em: one entry for each
# macro it calls, with Ember types in place of contract words.

unsafe overlay c "macros.h":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(macro_fn)
    fn SQUARE(x: i32) -> i32
    @ffi(macro_fn)
    fn SCALE(value: f64, by: f64) -> f64
    @ffi(macro_fn)
    fn POINT_SUM(p: Point) -> i32
    @ffi(macro_fn)
    fn PAIR_FIRST(p: *Pair) -> i64
    @ffi(macro_fn)
    fn IS_EVEN(n: u32) -> bool

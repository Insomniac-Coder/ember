# The overlay of ../reject_a_macro_entry_that_does_not_match_its_header.em: each entry is refused.

unsafe overlay c "macros.h":
    @ffi(macro_fn)
    fn CUBE(x: i32) -> i32
    @ffi(macro_fn)
    fn SQUARE(x: i32, y: i32) -> i32
    @ffi(macro_fn)
    fn SCALE(v: f64, by: f64) -> f64
    @ffi(macro_fn)
    fn POINT_SUM(p: String) -> i32
    @ffi(macro_fn)
    fn HIDDEN_ID(h: Hidden) -> i32
    @ffi(macro_fn)
    fn twice(x: i32) -> i32
    @ffi(macro_fn)
    fn LOG_ALL(x: i32) -> i32
    @ffi(macro_fn)
    fn IS_EVEN(n: u32) -> bool
    @ffi(macro_fn)
    fn IS_EVEN(n: u32) -> bool

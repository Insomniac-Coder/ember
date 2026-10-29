# An overlay of `layers.h` as a dependency would write it, listed first by the FFI-30c tests:
# `layers_scale` is left unsafe, `Size` gains a second name. It lies in the tests' own package, so it
# records no header hash (`[FFI-55]`).

unsafe overlay c "layers.h":  # SAFETY(ffi): each entry matches its declaration in the header
    unsafe fn layers_scale(value, by)
    fn layers_add(a, b)
    rename Size as Extent
    @ffi(macro_fn)
    fn LAYERS_TWICE(x: i32) -> i32

# A dependency's overlay of `layers.h`, listed first by the FFI-30c tests: `layers_scale` is left
# unsafe, `Size` gains a second name.

unsafe overlay c "layers.h":  # SAFETY(ffi): each entry matches its declaration in the header
    fn layers_scale(value, by) -> unsafe
    fn layers_add(a, b)
    rename Size as Extent
    @ffi(macro_fn)
    fn LAYERS_TWICE(x: i32) -> i32

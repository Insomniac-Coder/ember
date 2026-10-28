# An overlay listed after ./theirs.em by ../reject_overlays_that_clash.em: each entry but one is
# refused.

unsafe overlay c "layers.h":  # SAFETY(ffi): each entry matches its declaration in the header
    fn layers_add(a, b)
    override fn size_area(s)
    @ffi(macro_fn)
    fn LAYERS_TWICE(x: i64) -> i64
    rename Size as Dims
    override fn layers_scale(value, by)
    override fn layers_scale(value, by)
    type Size
    enum Size

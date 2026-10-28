# The package's own overlay of `layers.h`, listed after ./theirs.em: it replaces `layers_scale`'s
# entry with `override`, and adds what the first did not state.

unsafe overlay c "layers.h":  # SAFETY(ffi): each entry matches its declaration in the header
    override fn layers_scale(value, by)
    fn size_area(s)
    rename layers_add as add

# The overlay of ../reject_facts_an_overlay_does_not_state.em: one entry leaves a pointer without
# facts, another marks a fact still unknown.

unsafe overlay c "widgets.h":  # SAFETY(ffi): each entry matches its declaration in the header
    fn widget_sum(items, n)
    fn widget_fill(items: borrowed TODO(count), n, value)
    fn widget_add(a, b)

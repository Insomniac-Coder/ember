# The overlay of the FFI-2a tests: no entry says whether C writes; `const` does.

unsafe overlay c "buffers.h":
    fn buffers_first(items: borrowed count(n), n)
    fn buffers_zero(items: borrowed count(n), n)
    fn buffers_pick(items: borrowed count(n), n) -> borrowed one from(items)

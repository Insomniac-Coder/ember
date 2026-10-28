# The overlay of ../accept_overrides_beyond_an_overlay_entry_are_passed_on.em: an entry from
# before ODR-221 names one virtual; the program's classes override more.

unsafe overlay cpp "stages.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(trampoline, virtuals=["tick"])
    type Stage

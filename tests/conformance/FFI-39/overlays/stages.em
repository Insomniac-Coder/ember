# The overlay of ../accept_overrides_beyond_an_overlay_entry_are_passed_on.em: an entry from
# before ODR-221 names one virtual; the program's classes override more.

unsafe overlay cpp "stages.hpp":
    @ffi(trampoline, virtuals=["tick"])
    type Stage

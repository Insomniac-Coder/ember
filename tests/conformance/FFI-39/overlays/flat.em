# The overlay of ../reject_a_trampoline_base_without_a_virtual_destructor.em.

unsafe overlay cpp "panels.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(trampoline, virtuals=["depth"])
    type Flat

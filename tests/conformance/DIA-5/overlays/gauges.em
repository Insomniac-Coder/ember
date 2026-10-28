# The overlay of ../reject_a_cpp_base_error_names_the_class_declaration.em.

unsafe overlay cpp "gauges.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(trampoline, virtuals=["read"])
    type Gauge

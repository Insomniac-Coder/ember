# An overlay for another header: ../reject_an_overlay_for_another_header.em imports it for tally.h.

unsafe overlay c "counter.h":  # SAFETY(ffi): each entry matches its declaration in the header
    fn tally_scale(value, by)

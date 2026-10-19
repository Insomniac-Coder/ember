# An overlay for another header: reject_an_overlay_for_another_header.em imports it for tally.h.

unsafe overlay c "counter.h":
    fn tally_scale(value, by)

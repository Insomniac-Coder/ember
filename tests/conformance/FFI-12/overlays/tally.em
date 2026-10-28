# The overlay of ../accept_an_overlay_matching_its_header.em: every entry matches `tally.h`.

unsafe overlay c "tally.h":  # SAFETY(ffi): each entry matches its declaration in the header
    fn tally_sum(items: borrowed count(count), count)
    fn tally_scale(value, by)

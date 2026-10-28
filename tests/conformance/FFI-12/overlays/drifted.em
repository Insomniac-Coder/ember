# The overlay of ../reject_an_overlay_that_drifted_from_its_header.em: each entry has drifted.

unsafe overlay c "tally.h":  # SAFETY(ffi): each entry matches its declaration in the header
    fn tally_total(items: borrowed count(count), count)
    fn tally_sum(items: borrowed count(n), n)
    fn tally_scale(value)
    type Counter
    hide tally_reset

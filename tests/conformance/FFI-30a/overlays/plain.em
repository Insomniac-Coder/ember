# One overlay of pair.h for the FFI-30a tests: a function renamed and made safe.

unsafe overlay c "pair.h":  # SAFETY(ffi): each entry matches its declaration in the header
    fn pair_sum(p)
    rename pair_sum as total

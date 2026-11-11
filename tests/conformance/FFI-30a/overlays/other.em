# Another overlay of pair.h: the same function under another name.

unsafe overlay c "pair.h":  # SAFETY(ffi): each entry matches its declaration in the header
    fn pair_sum(p)
    rename pair_sum as add_up

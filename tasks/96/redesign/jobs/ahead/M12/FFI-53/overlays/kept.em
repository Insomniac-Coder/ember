# The overlay of ../*.em: one callback C keeps, once, until its token ends, or on the thread that
# kept it, unregistered by `unkeep` (which this test's C gets wrong on purpose).

unsafe overlay c "kept.h":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(destroys(unkeep))
    fn keep(cb: callback(retained, user_data=data), data)
    @ffi(destroys(unkeep))
    fn keep_once(cb: callback(once, user_data=data), data)
    @ffi(destroys(unkeep))
    fn keep_here(cb: callback(retained, user_data=data) threads(creator), data)
    fn fire(value)
    fn fire_elsewhere(value)
    fn tagged()
    hide unkeep

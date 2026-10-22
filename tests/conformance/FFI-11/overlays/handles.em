# The overlay of ../accept_an_opaque_handle_is_borrowed_as_one.em.

unsafe overlay c "handles.h":
    @ffi(destroys(counter_free))
    type Counter

    fn counter_new(start) -> returns_owned(destructor=counter_free) one
    fn counter_get(counter: borrowed one)
    fn counter_add(counter: borrowed one, n)
    fn counter_same(counter: borrowed one) -> borrowed one from(counter)
    fn counter_same_mut(counter: borrowed one) -> borrowed one exclusive from(counter)
    fn counter_get_or(counter: nullable borrowed one, fallback)
    fn counter_add_if(counter: nullable borrowed one, n)
    fn counter_freed()
    hide counter_free

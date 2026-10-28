# The overlay of ../reject_a_view_cannot_outlive_what_it_borrows.em: the constructor and `first`
# (`from(self)`), so that the test does not share a module text with another.

unsafe overlay cpp "views.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    fn new()
    fn first() -> from(self)

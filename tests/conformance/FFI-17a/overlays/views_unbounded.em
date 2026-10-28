# The overlay of ../reject_a_view_result_without_a_lifetime_word.em: a view result whose entry
# states no lifetime.

unsafe overlay cpp "views.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    fn first()

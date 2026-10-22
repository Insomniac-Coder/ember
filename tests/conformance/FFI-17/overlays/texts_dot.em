# The overlay of ../run_fail_two_counted_views_of_unequal_lengths.em: `dot` alone, so that this
# program and ../accept_contract_words_reach_a_wrapped_function.em do not share one module text.

unsafe overlay cpp "texts.hpp":
    fn dot(left: borrowed count(count), right: borrowed count(count), count)

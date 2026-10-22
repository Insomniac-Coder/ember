# The overlay of ../accept_contract_words_reach_a_wrapped_function.em and
# ../run_fail_two_counted_views_of_unequal_lengths.em: the facts `texts.hpp` cannot state.

unsafe overlay cpp "texts.hpp":
    fn checksum(data: borrowed count(length), length)
    fn hex(data: borrowed count(length), length)
    fn ends(values: borrowed count(count), count)
    fn scale(value, factors: borrowed count(count), count)
    fn dot(left: borrowed count(count), right: borrowed count(count), count)
    fn new()
    fn add(values: borrowed count(count), count)
    fn fill(out: count(count) borrowed, count)
    fn total()
    fn spot() -> borrowed one from(self)
    fn parse(text: borrowed nul_terminated)
    fn append(text, bytes: borrowed count(count), count)
    fn tally_of(values: borrowed count(count), count)
    class Tally:
        fn fill(self, out: MutSpan[c_int]) -> Result[c_int, CppError]

# The overlay of ../accept_contract_words_reach_a_wrapped_function.em: the facts `texts.hpp` cannot
# state.

unsafe overlay cpp "texts.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    fn checksum(data: borrowed count(length), length)
    fn hex(data: borrowed count(length), length)
    fn ends(values: borrowed count(count), count)
    fn scale(value, factors: borrowed count(count), count)
    fn dot(left: borrowed count(count), right: borrowed count(count), count)
    fn tally_of(values: borrowed count(count), count)
    class Tally:
        fn new()
        fn add(values: borrowed count(count), count)
        @ffi(const)
        fn fill(out: count(count) borrowed, count)
        @ffi(const)
        fn total()
        @ffi(const)
        fn spot() -> borrowed one from(self)
        fn parse(text: borrowed nul_terminated)
        @ffi(const)
        fn append(text, bytes: borrowed count(count), count)

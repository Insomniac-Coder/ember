# The overlay of ../accept_out_and_status_words_reach_a_wrapped_method.em.

unsafe overlay cpp "statuses.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(status=Status, ok=ok, error=SplitError)
    enum Status
    class Splitter:
        fn new()
        @ffi(const)
        fn split(value, half: out one) -> status
        @ffi(const)
        fn decade(value, low: out one, high: out one)


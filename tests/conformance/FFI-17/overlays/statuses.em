# The overlay of ../accept_out_and_status_words_reach_a_wrapped_method.em.

unsafe overlay cpp "statuses.hpp":
    @ffi(status=Status, ok=ok, error=SplitError)
    enum Status

    fn new()
    fn split(value, half: out one) -> status
    fn decade(value, low: out one, high: out one)

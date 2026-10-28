# [CXX-1] — the corpus program's overlay: what `json_api.hpp`, and the part of nlohmann/json the
# program calls, cannot state. Each text crosses as a `Span[u8]`, its pointer and its length
# (`borrowed count(length)`, [FFI-11]), and each call is asserted safe ([FFI-2]).

unsafe overlay cpp "json_api.hpp":  # SAFETY(ffi): each entry matches its declaration in the header
    fn parse(text: borrowed count(length), length)
    fn count(document, key: borrowed count(length), length)
    fn is_object()
    fn size()
    fn dump(indent, indent_char, ensure_ascii, error_handler)

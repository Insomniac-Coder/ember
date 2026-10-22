#$ test: compile-fail
#$ rules: FFI-17a, FFI-11
#$ profiles: debug
#$ error[E5002]: `first` returns a view (a `std::string_view` or a `std::span`), whose lifetime only its entry states
# `[FFI-17a]` — a `std::string_view` or `std::span` result is a view, and "a result needs a lifetime
# word" (`[FFI-11]`: a result's lifetime is never derived): an entry that asserts the call safe
# without saying what the view borrows (`-> from(self)`) is `E5002`.

import cpp "views.hpp" with (project = "views", overlay = "overlays/views_unbounded.em")

fn main():
    pass

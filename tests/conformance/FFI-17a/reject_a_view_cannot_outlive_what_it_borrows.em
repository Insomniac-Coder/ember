#$ test: compile-fail
#$ rules: FFI-17a, FFI-11
#$ profiles: debug
# `[FFI-17a]`, `[FFI-11]` — a `std::string_view` result borrows what its entry's lifetime word
# names (`from(self)`): the object, so it cannot outlive the `ForeignBox` that owns it.

import cpp "views.hpp" with (project = "views", overlay = "overlays/views_first.em")

fn first_word() -> Span[u8]:
    words = cpp.Words.new().unwrap()
    return words.get().first()    #$ error[E3060]: `words` does not live long enough

fn main():
    println(first_word().len())

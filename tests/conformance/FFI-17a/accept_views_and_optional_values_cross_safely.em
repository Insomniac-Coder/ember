#$ test: run-pass
#$ rules: FFI-17a, FFI-11, FFI-2
#$ profiles: debug, development, release
#$ stdout: 3 6 -1 -3 6
#$ stdout: nothing to shout
#$ stdout: ember 3 2 13 hello hi
#$ stdout: 7 true 5 0 6.0 3.0
#$ stdout: 42 1
# `[FFI-17a]` — a `std::string_view` is a `Span[u8]`, a `std::span<const T>` a `Span[T]` and a
# `std::span<T>` a `MutSpan[T]`, zero-copy both ways: a parameter is the view's pointer and length,
# from which the thunk makes the view; a result is a view of the C++ object's elements, borrowing
# what the entry's lifetime word names (`[FFI-11]`: `from(self)`, `from(text)`, `from(static)`). A
# `std::optional<T>` of a `T` that maps by value is an `Option[T]`. A `std::unique_ptr<T>`
# parameter takes the `ForeignBox[T]` the caller gives up: C++ frees the object, once. The
# overlay's entries assert each call safe (`[FFI-2]`): no `unsafe`.

from std.ffi import c_int, CppError
import cpp "views.hpp" with (project = "views", overlay = "overlays/views.em")

fn main():
    values: Array[c_int] = [1, 2, 3]
    total = cpp.sum(values.as_span())
    cpp.negate(values.as_mut_span())
    shouted = cpp.shout_length("hello".as_bytes()).unwrap_or(0)
    println(cpp.vowels("ember burns".as_bytes()), total, values[0], values[2], shouted)
    match cpp.shout_length("".as_bytes()):
        Ok(length):
            println(length)
        Err(error):
            println(error.what)
    words = cpp.Words.new().unwrap()
    first = words.get().first()
    scores = words.get().scores()
    shown = f"{first.to_str().unwrap()} {scores[0]} {scores.len() - 1}"
    changed = words.get_mut().scores_mut()
    changed[0] = 10
    padded = "  hi  ".as_bytes()
    println(shown, words.get().total(), cpp.greeting().to_str().unwrap(), cpp.trim(padded).to_str().unwrap())
    scale = cpp.scaled(Some(2.0), 3.0).unwrap_or(0.0)
    plain = cpp.scaled(None, 3.0).unwrap_or(0.0)
    println(cpp.digit(55).unwrap_or(-1), cpp.digit(120).is_none(), cpp.or_zero(Some(5)), cpp.or_zero(None), scale, plain)
    box = cpp.make_box(41).unwrap()
    println(cpp.consume(box), cpp.boxes_destroyed())

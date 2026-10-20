#$ test: run-pass
#$ rules: FFI-17a, FFI-17e
#$ profiles: debug, development, release
#$ stdout: 5 30
#$ stdout: 32 6 160
#$ stdout: item 7 6
#$ stdout: caf? 4
#$ stdout: 9 2 1
#$ stdout: 11
# `[FFI-17a]` — the standard library's types cross as bridge types: a `std::vector<int>` result is
# a `CppVector[c_int]`, viewed in place (`span()`), copied (`to_array()`), grown (`push_back`)
# and passed back by `const&` (borrowed) or `&` (lent mutably); a `std::string` is a `CppString`,
# `to_str()` when its bytes are UTF-8 and `String.from(c)` copying them (an invalid byte becoming
# U+FFFD); a `std::shared_ptr<T>` is a `CppShared[T]`, whose `clone()` C++ counts; a
# `std::unique_ptr<T>` is a `ForeignBox[T]`. `[FFI-17e]` — the bridge types are `std.ffi`'s.

from std.ffi import CppVector, CppString, CppShared
import cpp "library.hpp"

fn main():
    unsafe:  # SAFETY(ffi): calls with values the bridge types own
        match cpp.squares(5):
            Ok(values):
                println(values.len(), cpp.total(values))
                cpp.double_all(values)
                values.push_back(100)
                println(values.span()[4], values.to_array().len(), cpp.total(values))
            Err(error):
                println(error.what)
        match cpp.label(7):
            Ok(text):
                match text.to_str():
                    Ok(view):
                        println(view, text.len())
                    Err(_):
                        println("not UTF-8")
            Err(error):
                println(error.what)
        match cpp.broken():
            Ok(text):
                copied = String.from(text)
                shown = copied.replace("\u{FFFD}", "?")
                println(shown, shown.len())
            Err(error):
                println(error.what)
        match cpp.shared_counter(9):
            Ok(shared):
                again = shared.clone()
                println(shared.get().value(), cpp.owners(again), again.get().value() - 8)
            Err(error):
                println(error.what)
        match cpp.unique_counter(11):
            Ok(boxed):
                println(boxed.get().value())
            Err(error):
                println(error.what)

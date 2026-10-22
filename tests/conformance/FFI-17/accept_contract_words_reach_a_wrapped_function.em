#$ test: run-pass
#$ rules: FFI-17, FFI-11, FFI-2, FFI-24, FFI-40
#$ profiles: debug, development, release
#$ stdout: 256 no bytes 010203fa
#$ stdout: 4 6 no values 30
#$ stdout: 15 3 15 17 too many slots 15
#$ stdout: 22 not a digit 010203fa!?
#$ stdout: 6 32
# `[FFI-17]`, `[FFI-11]` (D-808) — an overlay's contract words mean for a C++ function Ember reaches
# through a wrapper what they mean for a C function: a function that may throw (`[FFI-24]`, its
# `Result`) or a method (`[FFI-40]`) whose pointer and length are `borrowed count(n)` takes a
# `Span` (`MutSpan` through a non-`const` pointer), the length leaving its signature; a
# `nul_terminated` pointer is a `cstr`, and a pointer result `borrowed one from(self)` a reference
# borrowing the receiver. Beside them the wrapper still passes what the importer does: a `T&`, the
# object a `ForeignBox` or a bridge value holds, and a class by value or a bridge value as the
# result. The entries assert each call safe (`[FFI-2]`): no `unsafe`.

from std.ffi import c_int, CppError, CppString
import cpp "texts.hpp" with (overlay = "overlays/texts.em")

fn shown[T](owned result: Result[T, CppError]) -> String:
    match result:
        Ok(_):
            return "ok"
        Err(error):
            return f"{error.what}"

fn main():
    bytes: Array[u8] = [1, 2, 3, 250]
    empty: Array[u8] = []
    sum = cpp.checksum(bytes.as_span()).unwrap_or(0)
    match cpp.hex(bytes.as_span()):
        Ok(hex):
            println(sum, shown(cpp.checksum(empty.as_span())), String.from(hex))
        Err(error):
            println(error.what)
    values: Array[c_int] = [4, 5, 6]
    nothing: Array[c_int] = []
    value: c_int = 2
    factors: Array[c_int] = [3, 5]
    _ = cpp.scale(value, factors.as_span())
    match cpp.ends(values.as_span()):
        Ok(ends):
            println(ends.first, ends.last, shown(cpp.ends(nothing.as_span())), value)
        Err(error):
            println(error.what)
    tally = cpp.Tally.new()
    tally.get_mut().add(values.as_span())
    slots: Array[c_int] = [0, 0, 0]
    many: Array[c_int] = [0, 0, 0, 0, 0]
    filled = tally.get().fill(slots.as_mut_span()).unwrap_or(0)
    spot: ref c_int = tally.get().spot()
    println(tally.get().total(), filled, slots[0], slots[2], shown(tally.get().fill(many.as_mut_span())), spot)
    parsed = tally.get_mut().parse(c"7").unwrap_or(0)
    failed = shown(tally.get_mut().parse(c"x"))
    appended = cpp.hex(bytes.as_span()).unwrap()
    mark: Array[u8] = [33, 63]
    _ = tally.get().append(appended, mark.as_span())
    println(parsed, failed, String.from(appended))
    small: Array[c_int] = [1, 2, 3]
    match cpp.tally_of(small.as_span()):
        Ok(other):
            println(other.get().total(), cpp.dot(small.as_span(), values.as_span()).unwrap_or(0))
        Err(error):
            println(error.what)

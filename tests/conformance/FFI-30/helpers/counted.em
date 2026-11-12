# A module of ../accept_one_cpp_header_imported_by_two_modules.em (D-879): `classes.hpp` imported
# from a folder below, once as the program's own import is (the same module) and once with another
# overlay (a second module, whose `Counter` is the first's, members and all).

from std.ffi import c_int

import cpp "../classes.hpp" with (overlay = "overlays/classes.em") as same
import cpp "../classes.hpp" with (overlay = "overlays/classes_doubled.em") as other

pub fn made() -> c_int:
    counter = same.Counter.new()
    counter.add(2)
    again = other.Counter.new()
    again.add(5)
    return counter.count() * 10 + again.count()

pub fn twice(n: c_int) -> c_int:
    return other.doubled(n)

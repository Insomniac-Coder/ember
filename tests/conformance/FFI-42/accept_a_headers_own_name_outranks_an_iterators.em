#$ test: run-pass
#$ rules: FFI-42
#$ warning[W5002]: `Row.iter` is not imported from `named.hpp`: its iterator's name `Row_iter` is taken by another declaration of the header
#$ stdout: 4
# `[FFI-42]` — an iterable class's iterator is `<Class>_iter`, written once the header's own
# declarations have their names: the header's `Row_iter`, declared after `Row`, keeps its name, and
# `Row`'s iteration is the one not imported, saying why (`W5002`), as a macro whose name a
# declaration has is not.

import cpp "named.hpp"

fn main():
    println(cpp.Row_iter(at = 4).at)

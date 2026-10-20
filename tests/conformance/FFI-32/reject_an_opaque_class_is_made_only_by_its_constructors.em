#$ test: compile-fail
#$ rules: FFI-32
#$ warning[W5033]: `Counter.consume` is not imported
# `[FFI-32]` — an opaque C++ class cannot be made from Ember but through its imported constructors
# (`E5032`): `Counter.new(…)`, which returns the `ForeignBox` its destructor frees.

import cpp "classes.hpp"

fn main():
    counter = cpp.Counter()    #$ error[E5032]: `Counter` is an opaque C++ class

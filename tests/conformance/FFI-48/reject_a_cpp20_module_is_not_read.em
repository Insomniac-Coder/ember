#$ test: compile-fail
#$ rules: FFI-48
# `[FFI-48]` — a header that declares or imports a C++20 module is not read: a compiled module
# interface has no stable AST (`E5034`, naming the row); the headers are parsed instead.

import cpp "modular.hpp"    #$ error[E5034]: a C++20 module

fn main():
    pass

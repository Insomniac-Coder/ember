#$ test: compile-fail
#$ rules: FFI-6b, FFI-12
#$ error[E5010]: `macros.h` defines no function-like macro `CUBE`
#$ error[E5010]: `SQUARE` takes 1 parameters in `macros.h`, and the entry names 2
#$ error[E5010]: parameter `v` of `SCALE` is `value` in `macros.h`
#$ error[E5015]: `String` has no C type for the wrapper of macro `POINT_SUM`
#$ error[E5015]: `Hidden` has no C type for the wrapper of macro `HIDDEN_ID`
#$ error[E5010]: `twice` is a function of `macros.h` as well as a macro
#$ error[E5010]: parameter `x` of `LOG_ALL` is `...` in `macros.h`
#$ error[E5010]: a second entry for `IS_EVEN` in one overlay
#$ warning[W5001]: macro `twice` is not imported
#$ warning[W5001]: macro `SQUARE` is not imported
#$ warning[W5001]: macro `SCALE` is not imported
#$ warning[W5001]: macro `POINT_SUM` is not imported
#$ warning[W5001]: macro `PAIR_FIRST` is not imported
#$ warning[W5001]: macro `HIDDEN_ID` is not imported
#$ warning[W5001]: macro `LOG_ALL` is not imported
# `[FFI-6b]` (ODR-147), `[FFI-12]` — an `@ffi(macro_fn)` entry matches a function-like macro of the
# header by its name, parameter count and parameter names (`E5010`); a function the header
# declares under the macro's name takes an ordinary entry. Each type it writes is one the wrapper
# can declare in C: a scalar, `cstr`, the header's type, a raw pointer (`E5015`: a `String`, an
# opaque struct by value). A macro whose entry is refused stays `W5001`.

import c "macros.h" with (overlay = "overlays/drifted.em")

fn main():
    pass

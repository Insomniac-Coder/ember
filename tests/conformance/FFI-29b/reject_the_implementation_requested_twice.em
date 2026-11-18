#$ test: compile-fail
#$ rules: FFI-29b
# `[FFI-29b]` — two imports of one header that each request its implementation would define it
# twice: `E5014`.

# SAFETY: single.h's implementation defines two functions and runs nothing before `main`
unsafe import c "single.h" with (implementation = ["SINGLE_IMPLEMENTATION"])
# SAFETY: as above
unsafe import c "single.h" with (implementation = ["SINGLE_IMPLEMENTATION"], defines = ["SINGLE_OTHER"]) as other  #$ error[E5014]: `single.h`'s implementation (`SINGLE_IMPLEMENTATION`) is already compiled into this build

fn main():
    pass

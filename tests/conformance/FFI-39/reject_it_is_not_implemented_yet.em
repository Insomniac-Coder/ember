#$ test: compile-fail
#$ rules: FFI-39
# And the honest half. The production exists, and the C++ importer and the
# trampoline that give it meaning are Phase 7, so it is refused by name.
#
# A declaration that parses, is stored, and is then ignored by every later
# stage is the shape three defects in this compiler have already taken. For a
# base class it would be the worst of them: the program would link and be
# wrong.

@ffi(trampoline, virtuals=["OnUpdate"])
extern class cpp.RageV.Layer:      #$ error[E1010]: `extern class cpp.RageV.Layer` is not supported yet in this phase
    pass

fn main():
    println(1)

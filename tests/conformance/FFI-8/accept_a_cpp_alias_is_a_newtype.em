#$ test: run-pass
#$ rules: FFI-8
#$ profiles: debug, development, release
#$ stdout: 8081
#$ stdout: 0.75
#$ stdout: 8080
# `[FFI-8]` (ODR-249) — a C++ `using` alias and `typedef` are typedefs too: `@ffi(newtype)` in the
# overlay makes each a nominal type with its representation, which the thunks pass as C++ has it,
# a parameter, a result and a constructor's argument alike, and `as` converts it.

import cpp "cpp_newtypes.hpp" with (overlay = "overlays/cpp_newtypes.em")

fn main():
    port = cpp.next_port(8080 as cpp.port_t)
    println(port as u16)
    println(cpp.halve(1.5 as cpp.ratio_t) as f64)
    socket = cpp.Socket.new(8080 as cpp.port_t)
    println(socket.get().port() as u16)

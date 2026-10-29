#$ test: compile-fail
#$ rules: FFI-17e
#$ warning[L3015]: this lambda is written in unsafe code, so it is an `unsafe fn(…)` value
#$ warning[L3015]: `squares` is a C++ function the program calls in unsafe code
# `[FFI-17e]` — a bridge value is not `Send`: C++ makes a container no promise about other threads.

from std.ffi import c_int, CppVector
import std.thread
import cpp "library.hpp"

fn main():
    unsafe:  # SAFETY(ffi): a call with a value
        match cpp.squares(3):
            Ok(values):
                worker = thread.spawn(owned fn() => values.len())    #$ error[E7004]: the thread's function captures `values`, a `std.ffi.CppVector[i32]`, which is not Send
                println(worker.join())
            Err(_):
                pass

#$ test: compile-fail
#$ rules: FFI-10, FFI-8, EXP-6
# `[FFI-10]` — a foreign static of a record that is not `Copy` is a place in
# C's storage. Its fields are read and written where they are; the whole
# value is not moved out of C's storage when the record has a drop, for the
# global still holds it.

struct Handle:
    pub fd: i32

    fn drop(mut self):
        println(self.fd)

unsafe extern "C":
    static mut current: Handle

fn main():
    unsafe:  # SAFETY(ffi): declared as the C global is defined
        println(current.fd)
        current.fd = 4
        taken = current    #$ error[E3013]: cannot move out of a reference
        current.fd = taken.fd

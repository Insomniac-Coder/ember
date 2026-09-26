#$ test: parse-pass
#$ rules: FFI-10, FFI-8, FFI-49

unsafe extern "C":
    static raw_counter: i32
    static mut changing_counter: i32
    @ffi(immutable)
    static frozen_count: i32
    @ffi(immutable, link_name="aliased_count")
    static renamed_count: i32

fn main():
    unsafe:
        println(raw_counter)
        println(changing_counter)
    println(frozen_count)
    println(renamed_count)

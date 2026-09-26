unsafe extern "C":
    @ffi(param(input, borrowed, one), result(borrowed, one, from(static)), link_name="ffi_static_result_c")
    pub safe fn static_result(input: ref i32) -> ref i32

extern "C" fn ffi_static_result_c(input: *mut i32) -> *mut i32:
    unsafe:
        number: *mut i32 = alloc[i32](1)
        write(number, 0, 40 + read(input, 0))
        return number

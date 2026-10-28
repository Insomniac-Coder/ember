unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(data, borrowed, count(n)), link_name="ffi_counted_probe")
    pub safe fn counted(data: Span[u8], n: usize) -> i32

pub extern "C" fn ffi_counted_probe(data: *u8, n: usize) -> i32:
    unsafe:  # SAFETY(ffi): each foreign call matches its C declaration
        return read(data, 0) as i32 + n as i32

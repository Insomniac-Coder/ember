#$ test: run-pass
#$ rules: FFI-10, FFI-8
#$ profiles: debug, development, release
#$ stdout: true

unsafe extern "C":
    type Handle
    @ffi(link_name="opaque_is_null")
    fn probe(p: *Handle) -> bool

pub extern "C" fn opaque_is_null(p: *Handle) -> bool:
    return p.is_null()

fn main():
    unsafe:  # SAFETY(ffi): each foreign call matches its C declaration
        println(probe(null[*Handle]()))

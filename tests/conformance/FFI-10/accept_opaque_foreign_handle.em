#$ test: run-pass
#$ rules: FFI-10, FFI-8
#$ profiles: debug, release, shipping
#$ stdout: true

unsafe extern "C":
    type Handle
    @ffi(link_name="opaque_is_null")
    fn probe(p: *Handle) -> bool

pub extern "C" fn opaque_is_null(p: *Handle) -> bool:
    return p.is_null()

fn main():
    unsafe:
        println(probe(null[*Handle]()))

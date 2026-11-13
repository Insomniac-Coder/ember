#$ test: run-pass
#$ rules: FFI-33b, FFI-36a
#$ stdout: 16 8
# `[FFI-33b]` — since an `owns T` has one layout, every `owns T` foreign value of a type bound to its
# thread is two words, the address and the id of the thread that made it; one of a type bound to no
# thread is one word, its address (`[FFI-36a]`).

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(destroys(context_close), threads(creator))
    type Context
    @ffi(destroys(buffer_free))
    type Buffer
    @ffi(link_name="words_context_close")
    @safety("`context` is a context C made, not closed yet")
    unsafe fn context_close(context: *mut Context)
    @ffi(link_name="words_buffer_free")
    @safety("`buffer` is a buffer C made, not freed yet")
    unsafe fn buffer_free(buffer: *mut Buffer)

pub extern "C" fn words_context_close(context: *mut Context):
    pass

pub extern "C" fn words_buffer_free(buffer: *mut Buffer):
    pass

fn main():
    println(size_of[owns Context](), size_of[owns Buffer]())

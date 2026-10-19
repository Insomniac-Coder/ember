#$ test: compile-pass
#$ rules: GRM-35, TIER-1
# `[GRM-35]` — an overlay of only `rename` and `hide` items states no fact
# about foreign code, so it needs no `unsafe`; one that states facts is an
# `unsafe overlay`, and checks clean as written.

overlay c "zlib.h":
    rename deflateInit_ as deflate_init
    hide zlibCompileFlags

unsafe overlay c "zlib.h":
    @ffi(handle)
    type z_stream
    @ffi(effects=[FFI])
    fn zlibVersion() -> borrowed nul_terminated from(static)
    fn adler32(adler, buf: nullable borrowed count(len), len)

fn main():
    pass

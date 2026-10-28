#$ test: compile-fail
#$ rules: FFI-10, FFI-8, TYP-15
# `[FFI-10]` — a foreign static holds a value C lays out. A record with a
# `str` field has no C representation; one with a `cstr` field has one, but
# `cstr` is not `Sync`, and a static must be (`[STA-1]`).

@derive(Copy)
struct HasView:
    text: str

struct HasCText:
    name: cstr
    width: i32

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    static foreign_view: HasView    #$ error[E5015]: `HasView` has no foreign representation
    @ffi(immutable)
    static label: HasCText    #$ error[E7002]: foreign static type `HasCText` is not Sync

fn main():
    pass

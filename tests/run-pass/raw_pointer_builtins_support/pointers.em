# The module of ../raw_pointer_builtins_in_any_module.em: `null[P]()` and `ref_to_ptr` outside the root.

pub fn none() -> *mut i32:
    return null[*mut i32]()

pub fn first(x: i32) -> i32:
    pointer = ref_to_ptr(ref x)
    unsafe:  # SAFETY: `x` is alive for the read
        return pointer.read()

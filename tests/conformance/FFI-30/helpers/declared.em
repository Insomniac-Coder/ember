# A module of reject_declarations_of_one_function_that_disagree.em: it declares `ffi30_add` with
# 8-byte integers, which the definition does not take.

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @safety("the caller passes what `ffi30_add` takes")
    pub unsafe fn ffi30_add(a: i64, b: i64) -> i64

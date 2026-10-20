# A module of reject_declarations_of_one_function_that_disagree.em: it declares `ffi30_add` with
# 8-byte integers, which the definition does not take.

unsafe extern "C":
    @safety("the caller passes what `ffi30_add` takes")
    pub fn ffi30_add(a: i64, b: i64) -> i64

#$ test: compile-pass
#$ rules: DIA-18, FFI-11c
#$ profiles: debug
# `[DIA-18]` — the line E5002 prints for `find` (in
# `reject_foreign_call_names_each_unknown_fact.em`) is a contract the
# declaration accepts as printed: each `TODO(…)` keeps the call `unsafe` until
# the fact replaces it.

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(data, TODO(ownership), TODO(count), TODO(nullable)), result(TODO(ownership), TODO(count), TODO(nullable), TODO(lifetime)))
    @safety("callers pass what the C function `find` requires")
    fn find(data: *u8, key: u8) -> *u8

fn main():
    pass

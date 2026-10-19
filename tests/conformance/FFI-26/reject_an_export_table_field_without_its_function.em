#$ test: compile-fail
#$ rules: FFI-26
#$ profiles: debug
# `[FFI-26]` — an exported table's fields are `extern` function pointers, each holding the
# function of its own name with exactly the field's type; the attribute names the table's C
# symbol and its protocol version.

pub extern "C" fn on_create(id: u64) -> i32:
    return 1

fn on_update(id: u64, dt: f32) -> i32:
    return 2

@export_table("Api", protocol=1)
struct Api:
    on_create: extern "C" fn(u64) -> i32
    on_update: extern "C" fn(u64, f32) -> i32 #$ error[E2020]: expected `extern "C" fn(u64, f32) -> i32`, found `fn(u64, f32) -> i32`
    on_destroy: extern "C" fn(u64) -> i32 #$ error[E1010]: no function `on_destroy` for the exported table's field
    count: u32 #$ error[E0104]: field `count` of an exported table is not an `extern` function pointer

@export_table("Unversioned") #$ error[E0104]: `@export_table` needs the table's C symbol and its protocol version
struct Unversioned:
    on_create: extern "C" fn(u64) -> i32

fn main():
    pass

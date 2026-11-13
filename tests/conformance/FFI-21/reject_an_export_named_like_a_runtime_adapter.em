#$ test: compile-fail
#$ rules: FFI-26, FFI-21
# `[FFI-26]` — a name beginning with the runtime's prefix (`[RT-5]`) is taken, so an export under one
# is refused even in the unsafe form (`E0104`): the names of the adapters the compiler makes for a
# function given to C as an `extern "C" fn` (`[FFI-21]`) are among them, and no export can replace one.

fn add_one(value: i32) -> i32:
    return value + 1

# SAFETY: nothing else in the process defines this name
@unsafe(export("ember_native_fn_value_0"))    #$ error[E0104]: `ember_native_fn_value_0` begins with `ember_`, the runtime's prefix (`[RT-5]`), and exporting `descriptor_spelling` under it could replace a runtime function for the whole process; choose another name
fn descriptor_spelling() -> i32:
    return 7

# SAFETY: nothing else in the process defines this name
@unsafe(export("ember_native_fn_c_entry_0"))    #$ error[E0104]: `ember_native_fn_c_entry_0` begins with `ember_`
fn adapter_spelling() -> i32:
    return 8

fn main():
    callback: extern "C" fn(i32) -> i32 = add_one
    println(callback(41), descriptor_spelling(), adapter_spelling())

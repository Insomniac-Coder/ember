#$ test: run-pass
#$ rules: FN-6, FFI-21, FFI-26
#$ profiles: debug, release, shipping
#$ stdout: 42
#$ stdout: 7
#$ stdout: 8
#$ stdout: 9

fn add_one(value: i32) -> i32:
    return value + 1

@export("ember_native_fn_value_0")
fn descriptor_spelling() -> i32:
    return 7

@export("ember_native_fn_c_entry_0")
fn adapter_spelling() -> i32:
    return 8

@export("ember_native_fn_descriptor")
fn descriptor_type_spelling() -> i32:
    return 9

fn main():
    callback: extern "C" fn(i32) -> i32 = add_one
    println(callback(41))
    println(descriptor_spelling())
    println(adapter_spelling())
    println(descriptor_type_spelling())

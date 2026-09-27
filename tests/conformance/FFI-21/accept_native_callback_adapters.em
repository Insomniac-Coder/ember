#$ test: parse-pass
#$ rules: FN-6, FFI-9, FFI-21, FFI-22

unsafe extern "C":
    static ember_native_fn_value_0: i32
    static ember_native_fn_c_entry_1: i32

@export("reserved_static_value")
fn read_reserved_globals() -> i32:
    unsafe:
        return ember_native_fn_value_0 + ember_native_fn_c_entry_1

struct Pair:
    left: i32
    right: i32

struct Large:
    first: i64
    second: i64
    third: i64

fn add_one(value: i32) -> i32:
    return value + 1

fn add_two(value: i32) -> i32:
    return value + 2

fn select_native(choice: i32) -> fn(i32) -> i32:
    if choice == 0:
        return add_one
    return add_two

fn pair_sum(value: Pair) -> i32:
    return value.left + value.right

fn large_sum(value: Large) -> i64:
    return value.first + value.second + value.third

fn pair_change(mut value: Pair) -> i32:
    value.left += 10
    return value.left + value.right

fn pair_create(value: i32) -> Pair:
    return Pair(left = value, right = value + 1)

fn invoke_nested(callback: extern "C" fn(i32) -> i32, value: i32) -> i32:
    return callback(value)

fn check_value(value: i32):
    if value != 42:
        panic("wrong void callback value")

fn native_address(value: Pair) -> *Pair:
    return ref_to_ptr(ref value)

@export("make_named")
fn named() -> extern "C" fn(i32) -> i32:
    return add_one

@export("make_inline")
fn inline_callback() -> extern "C" fn(i32) -> i32:
    return fn(value: i32) -> i32 => value + 3

@export("make_selected")
fn selected(choice: i32) -> extern "C" fn(i32) -> i32:
    native: fn(i32) -> i32 = select_native(choice)
    return native

@export("make_pair")
fn pair_callback() -> extern "C" fn(Pair) -> i32:
    return pair_sum

@export("make_large")
fn large_callback() -> extern "C" fn(Large) -> i64:
    return large_sum

@export("make_mut")
fn mut_callback() -> extern "C" fn(mut Pair) -> i32:
    return pair_change

@export("make_returned")
fn returned_callback() -> extern "C" fn(i32) -> Pair:
    return pair_create

@export("make_nested")
fn nested_callback() -> extern "C" fn(extern "C" fn(i32) -> i32, i32) -> i32:
    return invoke_nested

@export("make_void")
fn void_callback() -> extern "C" fn(i32):
    return check_value

@export("check_native_identity")
fn check_identity() -> i32:
    pair = Pair(left = 10, right = 20)
    native: fn(Pair) -> *Pair = native_address
    if native(pair) == ref_to_ptr(ref pair):
        return 1
    return 0

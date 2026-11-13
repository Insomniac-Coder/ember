#$ test: parse-pass
#$ rules: FN-6, FFI-9, FFI-21, FFI-22

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    static ember_native_fn_value_0: i32
    static ember_native_fn_c_entry_1: i32

# SAFETY: nothing else in the process defines this name
@unsafe(export("reserved_static_value"))
fn read_reserved_globals() -> i32:
    unsafe:  # SAFETY(ffi): each foreign call matches its C declaration
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

fn pair_change(ref mut value: Pair) -> i32:
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

# SAFETY: nothing else in the process defines this name
@unsafe(export("make_named"))
fn named() -> extern "C" fn(i32) -> i32:
    return add_one

# SAFETY: nothing else in the process defines this name
@unsafe(export("make_inline"))
fn inline_callback() -> extern "C" fn(i32) -> i32:
    return fn(value: i32) -> i32 => value + 3

# SAFETY: nothing else in the process defines this name
@unsafe(export("make_selected"))
fn selected(choice: i32) -> extern "C" fn(i32) -> i32:
    native: fn(i32) -> i32 = select_native(choice)
    return native

# SAFETY: nothing else in the process defines this name
@unsafe(export("make_pair"))
fn pair_callback() -> extern "C" fn(Pair) -> i32:
    return pair_sum

# SAFETY: nothing else in the process defines this name
@unsafe(export("make_large"))
fn large_callback() -> extern "C" fn(Large) -> i64:
    return large_sum

# SAFETY: nothing else in the process defines this name
@unsafe(export("make_mut"))
fn mut_callback() -> extern "C" fn(ref mut Pair) -> i32:
    return pair_change

# SAFETY: nothing else in the process defines this name
@unsafe(export("make_returned"))
fn returned_callback() -> extern "C" fn(i32) -> Pair:
    return pair_create

# SAFETY: nothing else in the process defines this name
@unsafe(export("make_nested"))
fn nested_callback() -> extern "C" fn(extern "C" fn(i32) -> i32, i32) -> i32:
    return invoke_nested

# SAFETY: nothing else in the process defines this name
@unsafe(export("make_void"))
fn void_callback() -> extern "C" fn(i32):
    return check_value

# SAFETY: nothing else in the process defines this name
@unsafe(export("check_native_identity"))
fn check_identity() -> i32:
    pair = Pair(left = 10, right = 20)
    native: fn(Pair) -> *Pair = native_address
    if native(pair) == ref_to_ptr(ref pair):
        return 1
    return 0

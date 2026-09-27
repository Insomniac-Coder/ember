#$ test: parse-pass
#$ rules: FFI-9, FFI-26, FFI-33, FFI-33c

#! language "0.9.9"
#! threads creator

struct HeaderLeaf:
    value: i32

struct HeaderEnvelope:
    leaf: HeaderLeaf
    bias: i32

pub extern "C" fn header_score(envelope: HeaderEnvelope, callback: Option[extern "C" fn(i32) -> i32]) -> i32:
    match callback:
        Some(function):
            return function(envelope.leaf.value + envelope.bias)
        None:
            return 0

unsafe extern "C":
    type HeaderHandle

pub extern "C" fn header_handle_roundtrip(handle: *HeaderHandle) -> *HeaderHandle:
    return handle

pub extern "C" fn header_i128_identity(value: i128) -> i128:
    return value

@export("header_creator", threads=creator)
pub fn creator_score() -> i32:
    return 46

@export("header_any", threads=any)
pub fn any_score() -> i32:
    return 47

@export("header_main", threads=main)
pub fn main_score() -> i32:
    return 48

# This private native callback must not add implementation-only declarations
# to the public C header.
fn private_callback_factory() -> extern "C" fn(i32) -> i32:
    return fn(value: i32) -> i32 => value + 1

fn private_str_length(value: str) -> i32:
    return 0

#$ test: run-pass
#$ rules: FFI-26, GRM-8
# XVI.10's own example, which had no production to parse it until errata
# ERR-036 added `["extern" string_lit]` to `fn_header`. It DEFINES a function
# with the C ABI — as against an `extern` block, which DECLARES foreign ones —
# and its symbol is the name as written, because a host that cannot find
# `on_update` by that name has no use for it.
#$ assert-c: contains("int32_t on_update(")

@export("rv_script_on_update")
pub extern "C" fn on_update(entity: u64, dt: f32) -> i32:
    return 1

fn main():
    println(on_update(1, 0.5))
#$ stdout: 1

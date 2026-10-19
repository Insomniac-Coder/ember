#$ test: run-pass
#$ rules: FFI-26
#$ profiles: debug, development
#$ stdout: 3
#$ stdout: 11
#$ stdout: 21
#$ assert-c: contains("RvScriptApi = { on_create, on_update };")
#$ assert-c: contains("const uint32_t RvScriptApi_protocol_3 = 3u;")
# `[FFI-26]` — `@export_table("Name", protocol=N)` exports a constant of the struct under the
# C symbol `Name`, each field holding the function of its own name, and its protocol version
# as `Name_protocol_N`, a symbol whose name carries the number (ABI-1). The program reads the
# table back through its own symbol, as a host that loads the module through that one entry
# point does.

pub extern "C" fn on_create(id: u64) -> i32:
    return (id + 10) as i32

pub extern "C" fn on_update(id: u64, dt: f32) -> i32:
    return (id as f32 * dt) as i32

@export_table("RvScriptApi", protocol=3)
pub struct ScriptApi:
    on_create: extern "C" fn(u64) -> i32
    on_update: extern "C" fn(u64, f32) -> i32

unsafe extern "C":
    @ffi(immutable)
    static RvScriptApi: ScriptApi
    @ffi(immutable)
    static RvScriptApi_protocol_3: u32

fn main():
    println(RvScriptApi_protocol_3)
    println(RvScriptApi.on_create(1))
    println(RvScriptApi.on_update(42, 0.5))

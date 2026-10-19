#$ test: run-pass
#$ rules: FFI-26
#$ profiles: debug, development
#$ stdout: 11
#$ stdout: 21
#$ stdout: 7
#$ assert-c: contains("RvScriptApi_v3 = { on_create, on_update, door_destroyed };")
#$ assert-c: !contains("RvScriptApi_protocol")
# `[FFI-26]` — `@export_table("Name", protocol=N)` exports a constant of the struct under the C
# symbol `Name_vN`, its protocol version in its name (`[ABI-1]`, ODR-129): a host built for another
# version does not find it. A field written with a default holds the function the default names
# (ODR-128); a field without one holds the function of its own name. The program reads the table
# back through its own symbol, as a host that loads the module through that one entry point does.

pub extern "C" fn on_create(id: u64) -> i32:
    return (id + 10) as i32

pub extern "C" fn on_update(id: u64, dt: f32) -> i32:
    return (id as f32 * dt) as i32

pub extern "C" fn door_destroyed(id: u64) -> i32:
    return (id + 6) as i32

@export_table("RvScriptApi", protocol=3)
pub struct ScriptApi:
    on_create: extern "C" fn(u64) -> i32
    on_update: extern "C" fn(u64, f32) -> i32
    on_destroy: extern "C" fn(u64) -> i32 = door_destroyed

unsafe extern "C":
    @ffi(immutable)
    static RvScriptApi_v3: ScriptApi

fn main():
    println(RvScriptApi_v3.on_create(1))
    println(RvScriptApi_v3.on_update(42, 0.5))
    println(RvScriptApi_v3.on_destroy(1))

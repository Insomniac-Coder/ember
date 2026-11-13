#$ test: compile-fail
#$ rules: FFI-26
# `[FFI-26]` — a C symbol is one name for the whole process, and the compiler cannot see every name
# the process links, so both export attributes are written in their unsafe form; the plain form is
# `E3100`, whose message gives the form to write.

@export("game_on_update")    #$ error[E3100]: `@export("game_on_update")` gives `on_update` a C name the whole process shares, and the compiler cannot see whether the host or a library already defines it; to vouch for it, write `@unsafe(export("game_on_update"))` with a `# SAFETY:` note above it
pub extern "C" fn on_update(entity: u64, dt: f32) -> i32:
    return 0

pub extern "C" fn door_created(id: u64) -> i32:
    return 1

@export_table("RvScriptApi", protocol=3)    #$ error[E3100]: `@export_table("RvScriptApi", protocol=3)` gives `ScriptApi`'s table a C name the whole process shares, and the compiler cannot see whether the host or a library already defines it; to vouch for it, write `@unsafe(export_table("RvScriptApi", protocol=3))` with a `# SAFETY:` note above it
pub struct ScriptApi:
    on_create: extern "C" fn(u64) -> i32 = door_created

fn main():
    pass

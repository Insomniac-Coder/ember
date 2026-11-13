#$ test: run-pass
#$ rules: FFI-26, UNS-8
#$ stdout: 0 1
# `[FFI-26]` — the unsafe form carries a `# SAFETY:` note on the line above, saying why nothing else
# defines the name; without one it is `W3012`, and the program still builds. A note at the end of the
# attribute's line is not one (`[LEX-23]`).

# SAFETY: the host defines no other symbol named `game_on_update`
@unsafe(export("game_on_update"))
pub extern "C" fn on_update(entity: u64, dt: f32) -> i32:
    return 0

@unsafe(export("game_on_draw"))    #$ warning[W3012]: this `unsafe` has no safety note
pub extern "C" fn on_draw(entity: u64) -> i32:
    return 1

fn main():
    println(on_update(1, 0.5), on_draw(2))

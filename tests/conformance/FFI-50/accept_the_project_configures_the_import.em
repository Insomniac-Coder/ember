#$ test: run-pass
#$ rules: FFI-50, FFI-17
#$ warning[L3015]: `scaled` is a C++ function the program calls in unsafe code
#$ profiles: debug, development, release
#$ stdout: 107 112
# `[FFI-50]` — the manifest's `[cpp.<project>]` names the C++ project an `import cpp` is parsed and
# its thunks compiled in: its `standard`, `defines`, `include_paths` and `flags` (`engine`), or,
# with `cmake = { build_dir, target }`, the target's own as CMake's File API replies them (`built`:
# another scale, include folder and flag).

import cpp "project.hpp" with (project = "engine") as engine
import cpp "project.hpp" with (project = "built") as built

fn main():
    unsafe:  # SAFETY(ffi): scalar calls
        println(engine.scaled(2), built.scaled(2))

#$ test: compile-fail
#$ rules: FFI-50
# `[FFI-50]` — `project` names a `[cpp.<project>]` section of the manifest.

import cpp "project.hpp" with (project = "missing")    #$ error[E0104]: the manifest has no `[cpp.missing]` section

fn main():
    pass

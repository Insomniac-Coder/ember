#$ test: run-pass
#$ rules: FN-8
#$ stdout: before
# `[FN-8]` — `process.exit(code)` ends a script early.

import std.process as process

println("before")
process.exit(0)
println("after")

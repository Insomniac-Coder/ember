#$ test: compile-fail
#$ rules: GRM-2
#$ error[E0100]: imports come first: move `import std.math` above line 6
# An import below a file-scope statement of the entry file is `E0100`.

x = 1
import std.math
println(x)

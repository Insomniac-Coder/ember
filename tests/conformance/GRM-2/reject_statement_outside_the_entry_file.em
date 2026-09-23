#$ test: compile-fail
#$ rules: GRM-2
#$ error[E0100]: a statement at file scope outside the entry file
#$ help: move it into a function
# Only the entry file may hold statements at file scope; an imported module
# that does is rejected where its first statement stands.

from script_support.helper import answer

println(answer())

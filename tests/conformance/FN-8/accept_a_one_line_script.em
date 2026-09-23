#$ test: run-pass
#$ rules: FN-8, CLI-4
#$ profiles: debug, release, shipping
#$ stdout: hello
# "`println("hello")` on its own is a complete program" ([CLI-4]).

println("hello")

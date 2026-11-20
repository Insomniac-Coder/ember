# Clause TYP-46a.1: "So `thread_shared T using A` is `E7015` at the making line when the value made
# would hold a link to an instance that is not one any thread may call: a value that grows made in
# an `Arena` (`thread_shared [String("a")] using frame`, the list of verdict 165)"
#$ test: compile-fail
#$ rules: TYP-46a, THR-1, ALC-1, LT-4
# The spec's own case. No text in the spec for it: code only.

fn main():
    frame = Arena.with_capacity(64)
    names = thread_shared [String("a")] using frame    #$ error[E7015]
    println(names.len())

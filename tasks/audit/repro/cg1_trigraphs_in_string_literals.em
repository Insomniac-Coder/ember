# Audit reproducer for CG-1 (tasks/audit/TASKS.md).
# Expected: what??! / a??=b / ... verbatim
# Observed at 46f225a: what| (plus out-of-bounds read of the next literal)
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/cg1_trigraphs_in_string_literals.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

fn main():
    println("what??!")
    println("a??=b")
    println("tab\there")
    println("quote\"q")
    println("back\\slash")
    println("nul\0end")

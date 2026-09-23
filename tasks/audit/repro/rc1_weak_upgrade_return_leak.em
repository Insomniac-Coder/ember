# Audit reproducer for RC-1 (tasks/audit/TASKS.md).
# Expected: 7 and no leak
# Observed at 46f225a: 7; LSan: 32 bytes leaked
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/rc1_weak_upgrade_return_leak.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

class Token:
    value: i32

fn observe(owned weak: Weak[Token]) -> i32:
    match weak.upgrade():
        Some(owner):
            return owner.value
        None:
            return 0

fn main():
    strong = Token(7)
    w = Weak(strong)
    println(observe(w))

# Audit reproducer for RC-1 (tasks/audit/TASKS.md).
# Expected: 3, 103, 2 (destructor runs)
# Observed at 46f225a: 3, 2 (object leaks; drop never runs)
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/rc1_match_temp_early_return_leak.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

class Token:
    value: i32

    fn drop(mut self):
        println(100 + self.value)

fn make() -> Option[Token]:
    return Some(Token(3))

fn get() -> i32:
    match make():
        Some(owner):
            return owner.value
        None:
            return 0

fn main():
    println(get())
    println(2)

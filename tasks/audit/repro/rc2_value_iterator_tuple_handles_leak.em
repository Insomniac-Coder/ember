# Audit reproducer for RC-2 (tasks/audit/TASKS.md).
# Expected: 7, 8, then 107/108 (destructors) before 0
# Observed at 46f225a: 7, 8, 0 (both objects leak)
# Run: cargo run -p ember_driver --bin ember -- run tasks/audit/repro/rc2_value_iterator_tuple_handles_leak.em --profile debug|release|shipping
# Memory errors: ember build <file> --emit c > x.c && gcc -std=c11 -O0 -g -fsanitize=address,undefined -I runtime/ember_rt/include x.c runtime/ember_rt/src/ember_rt.c -lm

class Token:
    value: i32

    fn drop(mut self):
        println(100 + self.value)

type Pair = (Token, Token)

struct PairIter:
    done: bool

    fn next(mut self) -> Option[Pair]:
        if self.done:
            return None
        self.done = true
        return Some((Token(7), Token(8)))

fn consume(owned token: Token) -> i32:
    return token.value

fn main():
    for left, right in PairIter(false):
        println(consume(left))
        println(consume(right))
    println(0)

#$ test: run-pass
#$ rules: RC-1, RC-2e, CTL-1, CTL-2, FN-1, TYP-14, TYP-22
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 2
#$ stdout: 7

# Dynamic interface dispatch preserves an `owned` parameter mode, so a
# loop-yielded class handle is retained at the erased call boundary.
class Token:
    value: i32

interface Consume:
    fn consume(self, owned token: Token) -> i32

class Worker implements Consume:
    fn consume(self, owned token: Token) -> i32:
        return token.value

fn main():
    worker = Worker()
    consumer: ref dyn Consume = ref worker
    tokens: Array[Token] = Array[Token]()
    tokens.push(Token(7))
    for token in tokens:
        println(consumer.consume(token))

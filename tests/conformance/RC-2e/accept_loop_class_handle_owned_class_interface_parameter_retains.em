#$ test: run-pass
#$ rules: RC-1, RC-2e, CTL-1, CTL-2, FN-1, TYP-14, OBJ-2, DSP-3, RNG-4
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 1
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 0
#$ stdout: 7

# A one-word class-interface receiver follows the class TypeInfo dispatch path.
# Its `owned` parameter must retain a loop-yielded class handle at that call,
# just as the equivalent `ref dyn` receiver does.
# Range facts prove the loop's index in bounds (`[RNG-4]`), so the loop is
# not versioned: its body, and the retains in it, appear once.
class Token:
    value: i32

interface Consume:
    fn consume(self, owned token: Token) -> i32

class Worker implements Consume:
    fn consume(self, owned token: Token) -> i32:
        return token.value

fn dispatch(consumer: Consume, owned tokens: Array[Token]):
    for token in tokens:
        println(consumer.consume(token))

fn main():
    tokens: Array[Token] = Array[Token]()
    tokens.push(Token(7))
    dispatch(Worker(), tokens)

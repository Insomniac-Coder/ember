#$ test: run-pass
#$ rules: RC-1, RC-2e, CTL-1, CTL-2, FN-1, TYP-14, DSP-2
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 3
#$ stdout: 7

# Virtual dispatch preserves an `owned` parameter mode, so a loop-yielded
# class handle retains at the virtual call boundary. The third retain is the
# independent Worker-to-Consumer class upcast used to select that dispatch.
class Token:
    value: i32

open class Consumer:
    fn init(mut self):
        pass

    virtual fn consume(self, owned token: Token) -> i32:
        return token.value

class Worker(Consumer):
    fn init(mut self):
        super.init()

    override fn consume(self, owned token: Token) -> i32:
        return token.value

fn dispatch(consumer: Consumer, owned tokens: Array[Token]):
    for token in tokens:
        println(consumer.consume(token))

fn main():
    tokens: Array[Token] = Array[Token]()
    tokens.push(Token(7))
    dispatch(Worker(), tokens)

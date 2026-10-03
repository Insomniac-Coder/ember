#$ test: run-pass
#$ rules: RC-1, RC-2e, CTL-1, CTL-2, FN-1, TYP-14, DSP-2, RNG-4
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("ember_retain_plain((ember_obj_header*)") == 2
#$ assert-c-count: contains("ember_retain((ember_obj_header*)") == 0
#$ stdout: 7

# Virtual dispatch preserves an `owned` parameter mode, so a loop-yielded
# class handle retains at the virtual call boundary. The other retain is the
# independent Worker-to-Consumer class upcast used to select that dispatch;
# the fresh Token constructor temporary transfers directly into its Array.
# Range facts prove the loop's index in bounds (`[RNG-4]`), so the loop is
# not versioned: its body, and the retains in it, appear once.
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

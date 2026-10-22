#$ test: run-pass
#$ rules: TYP-16, ENM-1, MONO-1
#$ stdout: 42
#$ stdout: 7
#$ assert-c: contains(struct em_7MessageG2c2a47066b4fc729)
#$ assert-c: contains(struct em_7MessageGaf29ae0d2798f0fb)
#$ assert-c: contains(em_5Token4drop(&)

enum Message[T]:
    Value(value: T)

    fn into_value(owned self) -> T:
        match owned self:
            Message.Value(value): return value

struct Token:
    value: i32

    fn drop(mut self):
        println(self.value)

fn main():
    message: Message[i32] = Message[i32].Value(42)
    println(message.into_value())
    _token = Message[Token].Value(Token(7))

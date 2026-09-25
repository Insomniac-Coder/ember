#$ test: run-pass
#$ rules: TYP-16, ENM-1, MONO-1
#$ stdout: 42
#$ stdout: 7
#$ assert-c: contains(struct em_Message_i32)
#$ assert-c: contains(struct em_Message_Token)
#$ assert-c: contains(em_Token_drop(&)

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

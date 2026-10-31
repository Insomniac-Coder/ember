struct Data:
    value: i32

extend Data implements Default:
    static fn default() -> Data:
        return Data(0)

class Token:
    payload: Data

    fn drop():
        retained: Array[Token] = Array[Token]()
        retained.push(this)

fn main():
    _token = Token(Data(7))

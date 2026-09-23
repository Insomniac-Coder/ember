struct Data:
    value: i32

extend Data implements Default:
    fn default() -> Data:
        return Data(0)

class Token:
    payload: Data

    fn drop(mut self):
        retained: Array[Token] = Array[Token]()
        retained.push(self)

fn main():
    _token = Token(Data(7))

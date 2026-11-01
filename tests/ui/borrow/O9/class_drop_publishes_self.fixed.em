import std.mem

struct Data:
    value: i32

extend Data implements Default:
    static fn default() -> Data:
        return Data(0)

class Token:
    payload: Data

    fn drop():
        retained: Array[Data] = Array[Data]()
        retained.push(owns mem.take(ref mut this.payload))

fn main():
    _token = Token(Data(7))

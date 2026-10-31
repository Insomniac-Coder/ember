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
        retained.push(mem.take(this.payload))

fn main():
    _token = Token(Data(7))

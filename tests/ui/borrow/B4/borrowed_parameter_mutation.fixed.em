fn append_one(ref mut values: Array[i32]):
    values.push(1)

fn main():
    values: Array[i32] = Array[i32]()
    append_one(values)


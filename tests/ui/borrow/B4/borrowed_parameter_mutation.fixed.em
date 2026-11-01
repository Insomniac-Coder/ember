fn append_one(ref mut values: Array[i32]):
    values.push(owns 1)

fn main():
    values: Array[i32] = Array[i32]()
    append_one(ref mut values)


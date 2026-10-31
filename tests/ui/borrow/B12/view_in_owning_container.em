fn main():
    text = String.from("ann")
    values: Array[StrView] = Array[StrView]()
    values.push(text.as_str())
    println(values.len())

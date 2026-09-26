fn main():
    text = String.from("ann")
    values: Array[String] = Array[String]()
    values.push(String.from(text.as_str()))
    println(values.len())

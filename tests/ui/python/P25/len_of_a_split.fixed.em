fn main():
    line = "a,b,c"
    parts = line.split(",").collect[Array[str]]()
    println(parts.len())

fn main():
    line = "a,b,c"
    parts = line.split(",").collect[Array[StrView]]()
    println(parts.len())

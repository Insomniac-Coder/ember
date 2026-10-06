fn main():
    s: String = ""
    for i in 0..8000000:
        s.push_str("hello ")
        s.push_str("world ")
    count = 0
    for b in s.as_bytes():
        if b == 111:
            count = count + 1
    println(len(s.as_bytes()), count)

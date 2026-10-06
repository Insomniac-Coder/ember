fn main():
    total = 0
    for i in 1..50000001:
        total = total + (i * 7) % 13 + i // 3 - i % 5
    println(total)

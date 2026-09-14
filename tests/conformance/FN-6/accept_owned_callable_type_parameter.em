#$ test: run-pass
#$ rules: FN-2, FN-6, FN-6a, OWN-3, CLO-3, TST-19
#$ stdout: 9

struct Packet:
    value: i32

fn consume(owned packet: Packet) -> i32:
    return packet.value

fn apply(f: fn(owned Packet) -> i32, owned packet: Packet) -> i32:
    return f(packet)

fn main():
    operation: fn(owned Packet) -> i32 = consume
    println(apply(operation, Packet(9)))

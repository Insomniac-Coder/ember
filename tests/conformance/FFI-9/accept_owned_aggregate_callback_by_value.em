#$ test: run-pass
#$ rules: FFI-9
#$ profiles: debug
#$ stdout: 42

struct Packet:
    x: i32
    y: i32

pub extern "C" fn add(owned packet: Packet) -> i32:
    return packet.x + packet.y

fn main():
    callback: extern "C" fn(owned Packet) -> i32 = add
    println(callback(Packet(20, 22)))

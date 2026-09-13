#$ test: run-pass
#$ rules: OWN-6, OWN-3, TYP-11
#$ assert-c: contains("/* mem.forget */")
#$ assert-c: !contains("em_Resource_drop(&")
#$ assert-c: !contains("ember_vec_free")
#$ assert-c: !contains("_Alignof(em_Padded)")

import std.mem

struct Resource:
    pub value: i32

    fn drop(mut self):
        println(self.value)

struct Padded:
    pub byte: u8
    pub wide: u64

fn alignment[T]() -> usize:
    return mem.align_of[T]()

fn main():
    resource = Resource(99)
    mem.forget(resource)
    mem.forget(Resource(98))
    values: Array[i32] = Array[i32]()
    values.push(7)
    println(values.len())
    mem.forget(values)
    println(mem.size_of[Padded]())
    println(mem.align_of[Padded]())
    println(align_of[u16]())
    println(alignment[u32]())
#$ stdout: 1
#$ 16
#$ 8
#$ 2
#$ 4

#$ test: run-pass
#$ rules: TYP-22, IFC-1, HEAP-1
#$ profiles: debug, development, release
#$ stdout: 20
#$ stdout: 22
#$ assert-c: contains(static const struct em_vt_dyn_4Read em_vt_dyn_4Read_for_6Signal)
#$ assert-c: contains(static const struct em_vt_dyn_5Write em_vt_dyn_5Write_for_6Signal)
#$ assert-c: contains(em_vt_dyn_4Read_for_6Signal_slot0)
#$ assert-c: contains(em_vt_dyn_5Write_for_6Signal_slot0)

interface Read:
    fn read(self) -> i32

interface Write:
    fn write(self) -> i32

enum Signal implements Read, Write:
    Ready

    fn read(self) -> i32:
        return 20

    fn write(self) -> i32:
        return 22

fn main():
    reader: Box[dyn Read] = Box(Signal.Ready)
    writer: Box[dyn Write] = Box(Signal.Ready)
    println(reader.read())
    println(writer.write())

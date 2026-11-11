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
    fn read() -> i32

interface Write:
    fn write() -> i32

enum Signal implements Read, Write:
    Ready

    fn read() -> i32:
        return 20

    fn write() -> i32:
        return 22

fn main():
    reader: owns dyn Read = owns Signal.Ready
    writer: owns dyn Write = owns Signal.Ready
    println(reader.read())
    println(writer.write())

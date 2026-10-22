#$ test: run-pass
#$ rules: HR-19, HR-12
#$ stdout: 6
#$ assert-c: contains("ember_obj_header_v24, ember_reload_protocol_v1")
#$ assert-c: contains("int64_t em_4area__rl(")
# `reload = "bodies"` (this directory's manifest) is the first tier: functions are reached through
# thunks so their bodies can change, but no instance ever migrates, so objects keep the plain
# 24-byte header with no live-instance list.

class Box:
    w: int = 2
    h: int = 3

fn area(b: Box) -> int:
    return b.w * b.h

fn main():
    println(area(Box()))

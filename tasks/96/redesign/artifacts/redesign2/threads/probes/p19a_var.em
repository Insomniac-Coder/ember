# A19: the Python spelling, a module variable (proposed `var`).
var FRAME: int = 0
var LOG: Array[String] = []

fn tick():
    FRAME += 1
    LOG.push("tick".to_string())

fn main():
    tick()

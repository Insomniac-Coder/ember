# A19: Python's spelling exactly, an assignment at module level.
FRAME = 0
LOG: Array[String] = []

fn tick():
    global FRAME
    FRAME += 1
    LOG.push("tick".to_string())

fn main():
    tick()

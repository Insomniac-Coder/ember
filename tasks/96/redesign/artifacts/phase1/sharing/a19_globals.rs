// A19: a frame counter and a log every system can reach.
static mut FRAME: u64 = 0;
static LOG: Vec<String> = Vec::new();

pub fn tick() {
    FRAME += 1;
    LOG.push(format!("tick"));
}

struct Pair:
    pub x: i32
    pub y: i32

    fn set_y(mut self, value: i32):
        self.y = value

fn main():
    pair = Pair(1, 2)
    view: ref mut i32 = ref mut pair.x
    pair.y = 10
    println(view)


# A traffic light state machine with timed phases, a pedestrian button that shortens green, and a fault mode.

enum Light:
    Red
    RedAmber
    Green
    Amber
    FlashingAmber

enum Input:
    Tick
    ButtonPressed
    Fault
    Repaired

struct Controller:
    state: Light = Light.Red
    timer: int = 0
    waiting_pedestrian: bool = false

    fn duration(self) -> int:
        return match self.state:
            Red => 5
            RedAmber => 1
            Green => 8
            Amber => 2
            FlashingAmber => 0

    fn next_state(self) -> Light:
        return match self.state:
            Red => Light.RedAmber
            RedAmber => Light.Green
            Green => Light.Amber
            Amber => Light.Red
            FlashingAmber => Light.FlashingAmber

    fn change(mut self, to: Light):
        println(f"    {self.state!r} -> {to!r}")
        self.state = to
        self.timer = 0
        if to == Light.Red:
            self.waiting_pedestrian = false

    fn handle(mut self, input: Input):
        match input:
            Tick:
                if self.state == Light.FlashingAmber:
                    return
                self.timer += 1
                limit = self.duration()
                if self.state == Light.Green and self.waiting_pedestrian:
                    limit = min(limit, 3)
                if self.timer >= limit:
                    self.change(self.next_state())
            ButtonPressed:
                if self.state != Light.Red:
                    self.waiting_pedestrian = true
            Fault:
                self.change(Light.FlashingAmber)
            Repaired:
                if self.state == Light.FlashingAmber:
                    self.change(Light.Red)

fn main():
    c = Controller()
    script: Array[Input] = []
    for _ in 0..12:
        script.push(Input.Tick)
    script.push(Input.ButtonPressed)
    for _ in 0..6:
        script.push(Input.Tick)
    script.push(Input.Fault)
    script.push(Input.Tick)
    script.push(Input.Tick)
    script.push(Input.Repaired)
    for _ in 0..4:
        script.push(Input.Tick)

    for step, input in script.iter().enumerate():
        println(f"step {step:>2}: {input!r}")
        c.handle(input)
    println(f"final state: {c.state!r}, timer {c.timer}")

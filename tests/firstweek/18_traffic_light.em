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
    waiting_pedestrian: bool = False

    fn duration() -> int:
        return match this.state:
            Red => 5
            RedAmber => 1
            Green => 8
            Amber => 2
            FlashingAmber => 0

    fn next_state() -> Light:
        return match this.state:
            Red => Light.RedAmber
            RedAmber => Light.Green
            Green => Light.Amber
            Amber => Light.Red
            FlashingAmber => Light.FlashingAmber

    fn change(to: Light):
        println(f"    {this.state!r} -> {to!r}")
        this.state = to
        this.timer = 0
        if to == Light.Red:
            this.waiting_pedestrian = False

    fn handle(input: Input):
        match input:
            Tick:
                if this.state == Light.FlashingAmber:
                    return
                this.timer += 1
                limit = this.duration()
                if this.state == Light.Green and this.waiting_pedestrian:
                    limit = min(limit, 3)
                if this.timer >= limit:
                    this.change(this.next_state())
            ButtonPressed:
                if this.state != Light.Red:
                    this.waiting_pedestrian = True
            Fault:
                this.change(Light.FlashingAmber)
            Repaired:
                if this.state == Light.FlashingAmber:
                    this.change(Light.Red)

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

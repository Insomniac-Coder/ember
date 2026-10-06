fn main():
    count = 0
    for py in 0..1000:
        for px in 0..1000:
            x0 = (px as float) * 3.0 / 1000.0 - 2.0
            y0 = (py as float) * 2.0 / 1000.0 - 1.0
            x = 0.0
            y = 0.0
            i = 0
            while i < 200 and x * x + y * y <= 4.0:
                xt = x * x - y * y + x0
                y = 2.0 * x * y + y0
                x = xt
                i = i + 1
            count = count + i
    println(count)

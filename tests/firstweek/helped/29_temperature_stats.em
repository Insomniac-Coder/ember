# Statistics over a week of temperature readings: min, max, mean, median, standard deviation, daily highs and warm streaks.

struct Reading:
    day: String
    hour: int
    celsius: float

const RAW: str = "mon 6 11.5, mon 14 19.0, mon 22 13.2, tue 6 10.1, tue 14 21.4, tue 22 15.0, wed 6 12.2, wed 14 22.8, wed 22 16.9, thu 6 13.0, thu 14 hot, thu 22 17.5, fri 6 14.4, fri 14 24.1, fri 22 18.3, sat 6 15.9, sat 14 23.0, sat 22, sun 6 11.0, sun 14 17.2, sun 22 12.6"

fn parse_readings(text: str) -> Array[Reading]:
    readings: Array[Reading] = []
    for chunk in text.split(","):
        parts = chunk.trim().split(" ").collect[Array[str]]()
        if parts.len() != 3:
            println(f"bad reading: '{chunk.trim()}'")
            continue
        hour = parts[1].parse[int]()
        temp = parts[2].parse[float]()
        if hour.is_err() or temp.is_err():
            println(f"bad numbers in '{chunk.trim()}'")
            continue
        readings.push(Reading(day=(parts[0]).to_string(), hour=hour.unwrap(), celsius=temp.unwrap()))
    return readings

fn mean(xs: Array[float]) -> float:
    return sum(xs) / xs.len() as float

fn median(xs: Array[float]) -> float:
    sorted_xs = sorted(xs)
    n = sorted_xs.len()
    if n % 2 == 1:
        return sorted_xs[n // 2]
    return (sorted_xs[n // 2 - 1] + sorted_xs[n // 2]) / 2.0

fn std_dev(xs: Array[float]) -> float:
    m = mean(xs)
    variance = sum((x - m) ** 2 for x in xs) / xs.len() as float
    return variance.sqrt()

fn to_fahrenheit(c: float) -> float:
    return c * 9.0 / 5.0 + 32.0

fn main():
    readings = parse_readings(RAW)
    temps = [r.celsius for r in readings]
    println(f"{readings.len()} readings")
    println(f"min {temps.iter().min().unwrap():.1f}  max {temps.iter().max().unwrap():.1f}  mean {mean(temps):.2f}  median {median(temps):.2f}  sd {std_dev(temps):.2f}")

    days: Array[String] = []
    by_day: Map[String, Array[float]] = {}
    for r in readings:
        if r.day not in by_day:
            by_day[r.day] = []
            days.push(r.day.clone())
        by_day[r.day].push(r.celsius)
    for day in days:
        values = by_day[day].clone()
        high = values.iter().max().unwrap()
        println(f"{day}: high {high:5.1f}C / {to_fahrenheit(high):5.1f}F  low {values.iter().min().unwrap():5.1f}C")

    # longest run of consecutive readings above 18 degrees
    streak = 0
    best = 0
    for t in temps:
        streak = streak + 1 if t > 18.0 else 0
        best = max(best, streak)
    println(f"longest run of readings above 18C: {best}")

    hottest = readings[0].clone()
    for r in readings:
        if r.celsius > hottest.celsius:
            hottest = r.clone()
    println(f"hottest: {hottest.day} at {hottest.hour}:00 ({hottest.celsius}C)")

# Summarise an inline CSV of sales: units, revenue and average sale per region, and the best-selling product.

const DATA: str = """region,product,units,price
north,widget,12,2.50
south,widget,7,2.50
north,gadget,3,10.00
east,gizmo,20,1.25
south,gadget,5,10.00
east,widget,oops,2.50
north,gizmo,8,1.25
west,gadget
south,gizmo,11,1.25
"""

struct Sale:
    region: String
    product: String
    units: int
    price: float

    fn revenue(self) -> float:
        return self.units as float * self.price

@derive(Error)
enum CsvError:
    @error("expected 4 fields, got {count}")
    FieldCount(count: int)

struct RegionTotals:
    units: int = 0
    revenue: float = 0.0
    rows: int = 0

fn parse_line(line: str) -> Result[Sale]:
    fields = line.split(",").collect[Array[str]]()
    if fields.len() != 4:
        return Err(AnyError.from(CsvError.FieldCount(count=fields.len())))
    units = fields[2].trim().parse[int]()?
    price = fields[3].trim().parse[float]()?
    return Ok(Sale(region=fields[0].trim().to_string(), product=fields[1].trim().to_string(), units=units, price=price))

fn main():
    sales: Array[Sale] = []
    bad_lines = 0
    for i, line in DATA.lines().enumerate():
        if i == 0 or line.trim() == "":
            continue
        match parse_line(line):
            Ok(sale):
                sales.push(sale)
            Err(e):
                println(f"line {i + 1}: skipped ({e})")
                bad_lines += 1

    totals: Map[String, RegionTotals] = {}
    product_units: Map[String, int] = {}
    for sale in sales:
        if sale.region not in totals:
            totals[sale.region] = RegionTotals()
        totals[sale.region].units += sale.units
        totals[sale.region].revenue += sale.revenue()
        totals[sale.region].rows += 1
        product_units[sale.product] = product_units.get_or(sale.product, 0) + sale.units

    println(f"{"region":<8} {"units":>6} {"revenue":>10} {"avg":>8}")
    grand_total = 0.0
    for region, t in totals.items():
        avg = t.revenue / t.rows as float
        grand_total += t.revenue
        println(f"{region:<8} {t.units:>6} {t.revenue:>10.2f} {avg:>8.2f}")
    println(f"{"total":<8} {"":>6} {grand_total:>10.2f}")

    best = ""
    best_units = 0
    for product, units in product_units.items():
        if units > best_units:
            best = product
            best_units = units
    println(f"best seller: {best} ({best_units} units)")
    println(f"{sales.len()} rows used, {bad_lines} skipped")

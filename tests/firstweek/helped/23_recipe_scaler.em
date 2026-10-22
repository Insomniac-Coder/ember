# Scale a recipe to a different number of servings and print the amounts in tidy units.

enum Unit:
    Gram
    Kilogram
    Millilitre
    Litre
    Teaspoon
    Tablespoon
    Piece

struct Ingredient:
    name: String
    amount: float
    unit: Unit

struct Recipe:
    title: String
    servings: int
    ingredients: Array[Ingredient] = []

    fn scaled(self, servings: int) -> Recipe:
        factor = servings as float / self.servings as float
        result = Recipe(title=self.title.clone(), servings=servings)
        for ing in self.ingredients:
            result.ingredients.push(Ingredient(name=ing.name.clone(), amount=ing.amount * factor, unit=ing.unit))
        return result

fn tidy(amount: float, unit: Unit) -> (float, Unit):
    match unit:
        Teaspoon:
            if amount >= 3.0:
                return (amount / 3.0, Unit.Tablespoon)
        Gram:
            if amount >= 1000.0:
                return (amount / 1000.0, Unit.Kilogram)
        Millilitre:
            if amount >= 1000.0:
                return (amount / 1000.0, Unit.Litre)
        _:
            pass
    return (amount, unit)

fn format_amount(amount: float, unit: Unit) -> String:
    a, u = tidy(amount, unit)
    return match u:
        Gram => f"{a:.0f} g"
        Kilogram => f"{a:.2f} kg"
        Millilitre => f"{a:.0f} ml"
        Litre => f"{a:.2f} l"
        Teaspoon => f"{a:.1f} tsp"
        Tablespoon => f"{a:.1f} tbsp"
        Piece => f"{a.ceil():.0f}"

fn parse_ingredient(line: str) -> Result[Ingredient]:
    parts = line.split(";").collect[Array[str]]()
    if parts.len() != 3:
        return Err(AnyError.msg(f"expected 'name; amount; unit'"))
    name = parts[0].trim()
    amount = parts[1].trim().parse[float]()?
    other = parts[2].trim()
    unit = Unit.Piece
    if other == "g":
        unit = Unit.Gram
    elif other == "ml":
        unit = Unit.Millilitre
    elif other == "tsp":
        unit = Unit.Teaspoon
    elif other == "tbsp":
        unit = Unit.Tablespoon
    elif other != "pc":
        return Err(AnyError.msg(f"unknown unit '{other}'"))
    return Ok(Ingredient(name=name.to_string(), amount=amount, unit=unit))

const PANCAKES: str = """flour; 250; g
milk; 500; ml
eggs; 2; pc
sugar; 2; tsp
salt; 0.5; tsp
butter; 30; g
vanilla; a dash; tsp
lemon; 1; slice"""

fn main():
    recipe = Recipe(title="Pancakes", servings=4)
    for line in PANCAKES.lines():
        match parse_ingredient(line):
            Ok(ing) => recipe.ingredients.push(ing)
            Err(e) => println(f"skipping '{line}': {e}")

    for servings in [4, 2, 10, 25]:
        s = recipe.scaled(servings)
        println(f"{s.title} for {s.servings}:")
        for ing in s.ingredients:
            println(f"  {format_amount(ing.amount, ing.unit):>10}  {ing.name}")

# DiceXp - The RPG Dice eXpression interpreter
**DiceXp** is a library and command-line interface (CLI) app for parsing and rolling role-playing game style dice notations (e.g. "2d8+5").

There are two components to this crate: the CLI app and hte library module. The CLI app can be installed with `cargo install dicexp` and then used in the terminal to roll dice. The library provides a `DiceBag` struct which you initialize with a random number generator (RNG) from the [rand crate](https://crates.io/crates/rand) and then call `DiceBag.eval(...)` on each dice expression you wish to evaluate.

### Roll dice with standard RPG dice notation
**DiceXp** supports standard RPG dice notation, such as "1d20+3" or "3d6", where the number in front of the 'd' is the number of dice to roll and the number after the 'd' is the number of sides per die. You can use as many different kinds of dice as you like, such as "1d4+1d6+1d8-1d12".

### Arithmetic (+, -, *, /)
**DiceXp** supports basic arithmetic, specifically addition (+), subtraction (-), mutliplication (* or x), and division (/). Note that division is *integer division*, (unless computing the average, see below), meaning that it always rounds down to a whole number. **DiceXp** also supports nested parenthese. Thus all of the following are valid `dicexp` expressions:

* "1d4*1d20"
* "-3*(1+2)"
* "(1d10+5)x10+(1d20-10)"
* "4d6/10-5"
* "4(9(10/2-6-3*8+1*4/2)*8/2*5+4)*5+4(7+7-3*8)*3-10*(10)-1"

### Average, min, and max values
When **DiceXp** evaluates a dice expression, it also computes what the statistical average result of the dice rools would be, as well as the largest and smallest possible values (ie what if all dice rolled their maximum value or rolled all 1's).

## Alternatives to DiceXp
**DiceXp** was designed for standard dice notation and designed to handle relatively complex mathematical dice formulas. It does not support all RPG systems or dice rolling mechanics (eg roll two dice and keep the higher one). The best alternative to **DiceXp** is the [ndm](https://crates.io/crates/ndm) crate, which better supports table-top RPGs. Here's a side-by-side feature comparison to [ndm](https://crates.io/crates/ndm):

| Feature               | DiceXp | ndm |
|-----------------------|--------|-----|
| CLI App               | ✔     | ✔   |
| Roll standard dice    | ✔     | ✔   |
| Add and substract     | ✔     | ✔   |
| Multiply              | ✔     | ~   |
| Divide                | ✔     |     |
| Nested parentheses    | ✔     |     |
| Exploding dice        |        | ✔   |
| Keep N highest/lowest |        | ✔   |
| Average, min, and max | ✔      |     |
*~ ndm can only multiply dice by constants, not by other dice*

# DiceXp CLI App
The `dicexp` CLI app evaluates dice expressions provided as arguments on the commandline. For example:
```
$ dicexp 1d6x10+1d10
>>> 1d6x10+1d10 => 64
```

Multiple expressions may be provided at once:
```
$ dicexp 1d6 1d6 1d6 1d6
>>> 1d6 => 2
>>> 1d6 => 5
>>> 1d6 => 3
>>> 1d6 => 1
```

You can also print the range of possible values for the dice expressions with the `-r/--range` option and the average result with the `-a--average` option:
```
$ 
```

## Installation
To install the `dicexp` app, simply run the following command:
```bash
cargo install dicexp
```

## Usage
`dicexp [OPTIONS] [DICE_EXPRESSIONS]...`

### Options
 * `-a`, `--average`:        Show the average result for each dice expression
* `-r`, `--range`:           Show the minimum and maximum possible result for each dice expression
* `-q`, `--quiet`:           Show only the roll results and nothing more (incompatible with `-a/--average` and `-r/--range`
* `-s`, `--seed` <INTEGER>:  Optional seed for random number generator
* `-h`, `--help`:            Print help
* `-V`, `--version`:         Print version

# DiceXp Rust Library
The **DiceXp** library module provides three structs: `DiceBag`, `DiceRoll`, and `SyntaxError`. It also provides two utility functions to simplify instantiating a new RNG: `simple_rng(u64) -> StdRng` and `new_simple_rng() -> StdRng`.

### Struct DiceBag
Most of the time, you only need to use the `DiceBag` struct. `DiceBag` is instantiated with the RNG of your choice via `DiceBag::new(Rng)` and then is ready to use. To evaluate a dice expression, use the `eval(&str)` method, or to only evaluate the roll, min, max, or average result, use the matching `eval_...(&str)` method.

### Struct DiceRoll
This is returned by `DiceBag.eval(&str)` and holds the rolled total, as well as the min, max, and average for the expression.

### Struct SyntaxError
This error type is the `Err()` result whenever a `DiceBag` method fails to evaluate a dice expression.

## Examples

### Roll dice from user input
This example prompts the user to enter a dice expression, then evaluates it. If the roll total is greater than the expected average, it tells prints "That's a good roll!", but if not then it prints "That's not a good roll :("
```rust
fn main() {
	use crate::{DiceBag, new_simple_rng};
	use std::io;
	let mut dice_bag = DiceBag::new(new_simple_rng());
	println!("What would you like to roll? ");
	let mut input = String::new();
	io::stdin()
		.read_line(&mut input)
		.expect("failed to read from stdin");
	let dice_roll = dice_bag.eval(input.as_str()).expect("invalid dice expression");
	println!("You rolled a {}", dice_roll);
	if dice_roll.total >= dice_roll.average as i64 {
		println!("That's a good roll!");
	} else {
		println!("That's not a good roll :(");
	}
}
```

### Calculate average damage for various D&D weapons
This example compares the average damage for various weapons from the table-top RPG Dungeons & Dragons (aka D&D).
```rust
fn main() {
	use crate::{DiceBag, new_simple_rng};
	let mut dice_bag = DiceBag::new(new_simple_rng());
	let armory = vec![
		("great axe", "1d12"),
		("great sword", "2d6"),
		("heavy crossbow", "1d10+2"),
		("firebolt", "1d10"),
		("magic missile", "3d4+3")
	];
	println!("Average Damage:");
	for (name, dmg) in armory {
		println!("{}\t{}", dice_bag.eval_ave(dmg).unwrap(), name)
	}
}
```

#![deny(unused_must_use)]
use std::error::Error;
use core::fmt::{Debug, Formatter};
use std::collections::HashSet;
use std::num::{ParseFloatError, ParseIntError};
use rand;
use rand::RngCore;
#[cfg(serde)]
use serde;

/// The DiceBag struct is use to evaluate RPG dice notation expressions (eg "2d6+3")
/// # Example
/// ```
/// use dicexp::{DiceBag, new_simple_rng};
/// let mut dice_roller = DiceBag::new(new_simple_rng());
/// let dice_exp = "3d6-4";
/// let dice_roll = dice_roller.eval(dice_exp).expect("Error");
/// println!("Rolled {}: {}", dice_exp, dice_roll);
/// println!("The average result is {:.1}", dice_roll.average);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiceBag <R: rand::Rng + Clone + Debug + PartialEq + Eq>{
	rng: R
}

impl <R> DiceBag<R> where R: rand::Rng + Clone + Debug + PartialEq + Eq {
	/// Constructs a new `DiceBag` instance
	/// # Parameters
	/// * `rng`: A random number generator to use for rolling dice
	pub fn new(rng: R) -> Self { DiceBag{rng} }

	/// Rolls a number of dice and returns the result
	/// # Parameters
	/// * `n`: number of dice to roll
	/// * `d`: number of sides per die
	/// * `m`: number to add to the total
	pub fn roll(&mut self, n: u32, d: u32, m: i64) -> i64 {
		let mut total = 0i64;
		for _ in 0..n {
			let roll: u32 = self.rng.gen_range(1..=d);
			total += roll as i64;
		}
		return total + m;
	}

	/// Evaluates the given RPG dice notation expression
	/// # Parameters
	/// * `dice_expression`: An RPG dice notation expressions (eg "2d6+3")
	pub fn eval(&mut self, dice_expression: &str) -> Result<DiceRoll,SyntaxError>{
		Ok(DiceRoll{
			total: self.eval_as(dice_expression, EvalMode::Roll)?.parse::<i64>().map_err(|e| SyntaxError::from(e))?,
			min: self.eval_as(dice_expression, EvalMode::Minimum)?.parse::<i64>().map_err(|e| SyntaxError::from(e))?,
			max: self.eval_as(dice_expression, EvalMode::Maximum)?.parse::<i64>().map_err(|e| SyntaxError::from(e))?,
			average: self.eval_as(dice_expression, EvalMode::Average)?.parse::<f64>().map_err(|e| SyntaxError::from(e))?,
		})
	}

	fn eval_as(&mut self, dice_expression: &str, mode: EvalMode) -> Result<String, SyntaxError> {
		if dice_expression.starts_with("-") || dice_expression.starts_with("+"){
			// must start with a number or there will be problems
			let mut new_exp = String::from("0");
			new_exp.push_str(dice_expression);
			return self.eval_as(new_exp.as_str(), mode);
		}
		let mut x = String::new();
		// need to remove all whitespace, also using this opportunity to throw common exceptions
		let mut line = 1;
		let mut col = 0;
		for c in dice_expression.chars() {
			if c == '\n' {
				line += 1;
				col = 0;
			}
			col += 1;
			if c.is_whitespace() {continue;}
			match mode{
				// decimals allowed in average mode, but otherwise it is ints-only
				EvalMode::Average => {},
				_ => {
					if c == '.' {return Err(SyntaxError{
						msg: Some("Found '.', but decimal numbers are not supported (integer math only)".into()),
						line: Some(line),
						col: Some(col),
						cause: None,
					});}
				}
			}
			if c == '%' {
				// d% means d100
				x.push_str("100")
			} else if c == 'x' || c == 'X' {
				// multiplication old-school notation
				x.push('*');
			} else {
				x.push(c);
			}
		}
		// Parentheses
		while match x.find("(") {
			None => false,
			Some(i) => {
				let cpy =  x.clone();
				let x_str = cpy.as_str();
				let (open, close) = find_enclosure_from(x_str, i, '(', ')')?
					.ok_or_else(|| SyntaxError::from("Error: unmatched parentheses"))?;
				let middle = self.eval_as(&x_str[open+1 .. close-1], mode)?;
				let front = &x_str[0..open];
				let back = &x_str[close..];
				x.clear();
				x.push_str(front);
				x.push_str(middle.as_str());
				x.push_str(back);
				true
			}
		}{}
		// Dice
		while match x.find("d") {
			None => false,
			Some(i) => {
				let cpy =  x.clone();
				let x_str = cpy.as_str();
				let (start, end) = find_operator_params(x_str, i)?;
				let n = &x_str[start..i].parse::<u32>().map_err(|e| SyntaxError::from(e.clone()))?;
				let d = &x_str[i+1..end].parse::<u32>().map_err(|e| SyntaxError::from(e.clone()))?;
				let middle: String;
				match mode {
					EvalMode::Roll => middle = format!("{}", self.roll(*n, *d, 0)),
					EvalMode::Average => middle = format!("{:.1}", 0.5 * (1f64 + *d as f64)),
					EvalMode::Minimum => middle = "1".into(),
					EvalMode::Maximum => middle = format!("{}", d),
				}
				let front = &x_str[0..start];
				let back = &x_str[end..];
				x.clear();
				x.push_str(front);
				x.push_str(middle.as_str());
				x.push_str(back);
				true
			}
		}{}
		// multiply and divide
		while match find_one_of(x.as_str(), &['*', '/']) {
			None => false,
			Some(i) => {
				let cpy =  x.clone();
				let x_str = cpy.as_str();
				let op = &x_str[i..i+1];
				let (start, end) = find_operator_params(x_str, i)?;
				let middle: String;
				match mode {
					EvalMode::Average => {
						let left = &x_str[start..i].parse::<f64>().map_err(|e| SyntaxError::from(e.clone()))?;
						let right = &x_str[i+1..end].parse::<f64>().map_err(|e| SyntaxError::from(e.clone()))?;
						if op == "/" {
							middle = format!("{:.}", *left / *right);
						} else {
							middle = format!("{:.}", *left * *right);
						}
					}
					_ => {
						let left = &x_str[start..i].parse::<i64>().map_err(|e| SyntaxError::from(e.clone()))?;
						let right = &x_str[i+1..end].parse::<i64>().map_err(|e| SyntaxError::from(e.clone()))?;
						if op == "/" {
							middle = format!("{}", *left / *right);
						} else {
							middle = format!("{}", *left * *right);
						}
					}
				}
				let front = &x_str[0..start];
				let back = &x_str[end..];
				x.clear();
				x.push_str(front);
				x.push_str(middle.as_str());
				x.push_str(back);
				true
			}
		}{}

		// add and subtract
		while match find_one_of(x.as_str(), &['+', '-']) {
			None => false,
			Some(i) => {
				let cpy =  x.clone();
				let x_str = cpy.as_str();
				let op = &x_str[i..i+1];
				let (start, end) = find_operator_params(x_str, i)?;
				let middle: String;
				match mode {
					EvalMode::Average => {
						let left = &x_str[start..i].parse::<f64>().map_err(|e| SyntaxError::from(e.clone()))?;
						let right = &x_str[i+1..end].parse::<f64>().map_err(|e| SyntaxError::from(e.clone()))?;
						if op == "-" {
							middle = format!("{:.}", *left - *right);
						} else {
							middle = format!("{:.}", *left + *right);
						}
					}
					_ => {
						let left = &x_str[start..i].parse::<i64>().map_err(|e| SyntaxError::from(e.clone()))?;
						let right = &x_str[i+1..end].parse::<i64>().map_err(|e| SyntaxError::from(e.clone()))?;
						if op == "-" {
							middle = format!("{}", *left - *right);
						} else {
							middle = format!("{}", *left + *right);
						}
					}
				}
				let front = &x_str[0..start];
				let back = &x_str[end..];
				x.clear();
				x.push_str(front);
				x.push_str(middle.as_str());
				x.push_str(back);
				true
			}
		}{}
		// DONE!
		Ok(x)
	}

}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EvalMode {
	Roll, Average, Minimum, Maximum
}

/// The result of rolling the provided dice expression, including the average and minimum and
/// maximum possible results.
#[cfg_attr(feature="serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub struct DiceRoll {
	/// The amount rolled
	pub total: i64,
	/// Minimum possible result
	pub min: i64,
	/// Maximum possible result
	pub max: i64,
	/// Average result
	pub average: f64
}

impl core::fmt::Display for DiceRoll {
	/// core::fmt::Display implementation returns the total result
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		std::fmt::Display::fmt(&self.total, f)
	}
}

/// Error returns when a `DiceBag` fails to interpret or evaluate a dice expression
pub struct SyntaxError {
	pub msg: Option<String>,
	pub line: Option<u64>,
	pub col: Option<u64>,
	pub cause: Option<Box<dyn Error>>
}

impl SyntaxError{
	/// This function is used to make Debug and Display output the same
	fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "SyntaxError: ")?;
		match &self.msg {
			None => write!(f, "Failed to parse string")?,
			Some(s) => write!(f, "{}", s)?,
		}
		match &self.line {
			None => {}
			Some(s) => {
				write!(f, "; error on line {}", s)?;
				match &self.col {
					None => {}
					Some(c) => write!(f, ", column {}", c)?,
				}
			},
		}
		match &self.cause {
			None => {}
			Some(coz) => write!(f, "\n\tCaused by: {}", coz)?
		}
		Ok(())
	}
}

impl From<&str> for SyntaxError {
	fn from(value: &str) -> Self {
		SyntaxError{
			msg: Some(value.into()),
			line: None,
			col: None,
			cause: None,
		}
	}
}

impl From<ParseIntError> for SyntaxError {
	fn from(value: ParseIntError) -> Self {
		SyntaxError{msg: Some("Failed to parse string as integer".into()), line: None, col: None, cause: Some(Box::from(value)) }
	}
}

impl From<ParseFloatError> for SyntaxError {
	fn from(value: ParseFloatError) -> Self {
		SyntaxError{msg: Some("Failed to parse string as decimal number".into()), line: None, col: None, cause: Some(Box::from(value)) }
	}
}

impl Debug for SyntaxError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		self.print(f)
	}
}

impl core::fmt::Display for SyntaxError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		self.print(f)
	}
}

impl Error for SyntaxError {}


/// Creates a new random number generator (RNG) from the provided seed using the default
/// [rand crate](https://crates.io/crates/rand) `rand::rngs::StdRng` RNG
/// # Parameters
/// * `seed`: A 64-bit number to use as a seed
pub fn simple_rng(seed: u64) -> rand::rngs::StdRng {
	use rand::rngs::StdRng;
	use rand::SeedableRng;
	let mut seeder_rng_seed: <StdRng as SeedableRng>::Seed = <StdRng as SeedableRng>::Seed::default();
	let sub_seed: [u8; 8] = bytemuck::cast(seed);
	for i in 0..seeder_rng_seed.len() {
		seeder_rng_seed[i] = sub_seed[i % sub_seed.len()];
	}
	let mut seeder_rng = StdRng::from_seed(seeder_rng_seed);
	let mut rng_seed: <StdRng as SeedableRng>::Seed = <StdRng as SeedableRng>::Seed::default();
	seeder_rng.fill_bytes(&mut rng_seed);
	return StdRng::from_seed(rng_seed);
}

/// Creates a new random number generator (RNG) from the provided seed using the default
/// [rand crate](https://crates.io/crates/rand) `rand::rngs::StdRng` RNG, using the current system
/// millisecond timestamp as the RNG seed
pub fn new_simple_rng() -> rand::rngs::StdRng {
	use std::time::{SystemTime, UNIX_EPOCH};
	let time_seed = SystemTime::now().duration_since(UNIX_EPOCH)
		.expect("Invalid system time").as_millis() as u64;
	simple_rng(time_seed)
}


fn find_enclosure_from(text: &str, pos: usize, open: char, close: char) -> Result<Option<(usize, usize)>, SyntaxError> {
	let mut depth = 0;
	let slice = &text[pos..];
	let mut start_index = 0;
	for (i, c) in slice.char_indices() {
		if c == open {
			if depth == 0 {
				start_index = i + pos;
			}
			depth += 1;
		} else if c == close {
			depth -= 1;
			if depth == 0 {
				return Ok(Some((start_index, i+1)))
			}
		}
	}
	if depth > 0 {
		return Err(SyntaxError{msg: Some("Found '(' without matching ')'".into()), line: None, col: None, cause: None });
	}
	return Ok(None);
}

fn find_operator_params(text: &str, op_pos: usize) -> Result<(usize, usize), SyntaxError> {
	let front_slice = &text[0..op_pos];
	let back_slice = &text[op_pos+1..];
	let mut end = text.len();
	for (i, c) in back_slice.char_indices() {
		if !(c.is_digit(10) || c == '.') {end = op_pos+1+i; break;}
	}
	let mut start = 0;
	for (i, c) in front_slice.char_indices().rev() {
		if !(c.is_digit(10) || c == '.') {start = i+1; break;}
	}
	if start == op_pos || end == op_pos+1 {
		return Err(SyntaxError{
			msg: Some(format!("Missing numbers before or after operator {}", &text[op_pos..op_pos+1])),
			line: None,
			col: None,
			cause: None,
		});
	}
	Ok((start, end))
}

fn find_one_of(text: &str, chars: &[char]) -> Option<usize> {
	let mut set = HashSet::with_capacity(chars.len());
	for c in chars {set.insert(c);}
	for (i, c) in text.char_indices() {
		if set.contains(&c){
			return Some(i);
		}
	}
	return None;
}


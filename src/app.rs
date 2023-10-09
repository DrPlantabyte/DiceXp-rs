#![deny(unused_must_use)]
use std::error::Error;
use std::fmt::{Debug, Formatter};
use clap::{arg, Parser};
use dicexp::{DiceBag, new_simple_rng, simple_rng};
use rand;


/// CLI arguments struct (used with CLI parser module clap)
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
	/// Show the average result for each dice expression
	#[arg(short='a', long="average")]
	show_average: bool,
	/// Show the minimum and maximum possible result for each dice expression
	#[arg(short='r', long="range")]
	show_range: bool,
	/// Show only the roll results and nothing more (incompatible with -a/--average and -r/--range
	#[arg(short='q', long="quiet")]
	quiet: bool,
	/// Optional seed for random number generator
	#[arg(short='s', long="seed")]
	seed: Option<u64>,
	/// One or more RPG dice notation expressions to evaluate (eg "1d20+3")
	expressions: Vec<String>
}

/// Entry point for the CLI app
pub fn main() -> Result<(), Box<dyn Error>> {
	let args = Args::parse();
	sanity_check(&args)?;
	for output in run(args)? {
		println!("{}", output);
	}
	Ok(())
}

/// Runs the program, return a list of results for each expression
pub fn run(args: Args) -> Result<Vec<String>, Box<dyn Error>>  {
	let mut dice: DiceBag<rand::rngs::StdRng>;
	let mut results: Vec<String> = Vec::with_capacity(args.expressions.len());
	match args.seed {
		None => dice = DiceBag::new(new_simple_rng()),
		Some(seed) => dice = DiceBag::new(simple_rng(seed)),
	}
	for exp in &args.expressions {
		let mut output = String::new();
		let roll = dice.eval(exp.as_str())?;
		if ! args.quiet {
			output.push_str(exp.as_str());
			output.push_str(" => ");
		}
		output.push_str(format!("{}", roll.total).as_str());
		if ! args.quiet && (args.show_average || args.show_range) {
			output.push_str(" (");
			if args.show_range {
				output.push_str(format!("{}-{}", roll.min, roll.max).as_str());
			}
			if args.show_average && args.show_range {output.push_str(", ");}
			if args.show_average {
				output.push_str(format!("{:.1} ave.", roll.average).as_str());
			}
			output.push_str(")");
		}
		results.push(output);
	}
	Ok(results)
}

fn sanity_check(args: &Args) -> Result<(), Box<dyn Error>> {
	if args.quiet && (args.show_range || args.show_average) {
		return Err(InvalidArgumentError{msg: "Invalid arguments: -q/--quiet is not compatible with -a/--average and -r/--range".into()}.into());
	}
	Ok(())
}


/// Error returned when there's a bad CLI argument
#[derive(Clone)]
pub struct InvalidArgumentError {
	pub msg: String
}

impl InvalidArgumentError{
	/// This function is used to make Debug and Display output the same
	fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "InvalidArgumentError: {}", self.msg)?;
		Ok(())
	}
}

impl Debug for InvalidArgumentError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		self.print(f)
	}
}

impl core::fmt::Display for InvalidArgumentError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		self.print(f)
	}
}

impl Error for InvalidArgumentError {}


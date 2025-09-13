use regex::Regex;

/// Defines a token with a Regex pattern and a constructor function
#[derive(Debug, Clone)] 
pub struct TokenDef<T> {
	pub pattern: Regex,
	pub constructor:  fn(&str) -> T,
}

pub fn regex_lexer<T>(patterns: impl IntoIterator<Item = TokenDef<T>>, text: &str) -> Vec<T>
{
	let sequence: Vec<TokenDef<T>> = patterns.into_iter().collect();
	let super_pattern = join_patterns(&sequence);
	todo!()
}

fn join_patterns<T>(patterns: &Vec<TokenDef<T>>) -> String {
	let mut super_pattern = String::new();
	for i in  0..patterns.len() {
		let pat =  &patterns[i];
		let name = format!("T{}", i);
		let named_capture_group = format!("(?P<{name}>{})", pat.pattern.as_str());
		if i > 0 {
			super_pattern += "|";
		}
		super_pattern += &named_capture_group;
	}
	// add syntax error token
	super_pattern += "|(?P<UNMATCHED>.+)";
	return super_pattern;
}

#[test]
fn test_join_patterns() {
	let token_constructor = |s: &str| String::from(s);
	let toks = vec![
		TokenDef{pattern: Regex::new(r"\d+").unwrap(), constructor: token_constructor},
		TokenDef{pattern: Regex::new(r"\w+").unwrap(), constructor: token_constructor},
		TokenDef{pattern: Regex::new(r"\s+").unwrap(), constructor: token_constructor},
	];
	let result = join_patterns(&toks);
	println!("join_patterns({:?}) -> {:?}", toks, result);
	let text = "Bob and 23 birds.";
	let captures = Regex::new(result.as_str()).unwrap().captures_iter(text).collect::<Vec<_>>();
	println!("text: {}", text);
	for cap in captures {
		let (name, groups) = cap.extract();
		println!("capture '{}': {:?}", name, groups.iter().collect::<Vec<_>>());
	}
	todo!()
}
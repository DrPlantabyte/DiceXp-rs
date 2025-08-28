use rand::RngCore;
use dicexp;

#[test]
fn basic_arithmatic(){
	let mut dice_bag = dicexp::new_dice_bag_from_seed(12345);
	let result = dice_bag.eval("4(9(10/2-6-3*8+1*4/2)*8/2*5+4)*5+4(7+7-3*8)*3-10*(10)-1").unwrap();
	let val = -82941;
	let expected = dicexp::DiceRoll{total: val, min: val, max: val, average: val as f64};
	assert_eq!(result, expected);
}

#[test]
fn single_die() {
	let mut dice_bag = dicexp::new_dice_bag_from_seed(12345);
	let mut sum = 0;
	let reps = 1000;
	for _ in 0..reps {
		let result = dice_bag.eval("1d6").unwrap();
		assert_eq!(result.average, 3.5, "wrong average");
		assert_eq!(result.min, 1, "wrong min");
		assert_eq!(result.max, 6, "wrong max");
		assert!(result.total >= result.min && result.total <= result.max, "total out of expected range");
		sum += result.total;
	}
	let mut actual_ave = sum as f64 / reps as f64;
	assert!(actual_ave > 3.5*0.9 && actual_ave < 3.5*1.1, "average of {} rolls too far from the expected mean", reps);
}

#[test]
fn basic_usage() {
	let mut dice_bag = dicexp::new_dice_bag_from_seed(12345);
	for _ in 0..1000 {
		let result = dice_bag.eval("(1d4*1d4) / 2 + 10 - 100").unwrap();
		assert_eq!(result.average, 2.5*2.5 / 2. + 10. - 100.);
		assert_eq!(result.min, -90);
		assert_eq!(result.max, -82);
		assert!(result.total >= result.min && result.total <= result.max);
	}
}

#[test]
fn percent_dice() {
	let mut dice_bag = dicexp::new_dice_bag_from_seed(12345);
	let mut sum = 0;
	let reps = 1000;
	for _ in 0..reps {
		let result = dice_bag.eval("(1d%-1)*10+(1d10-1)").unwrap();
		assert_eq!(result.average, 499.5, "wrong average");
		assert_eq!(result.min, 0, "wrong min");
		assert_eq!(result.max, 999, "wrong max");
		assert!(result.total >= result.min && result.total <= result.max, "total out of expected range");
		sum += result.total;
	}
	let mut actual_ave = sum as f64 / reps as f64;
	assert!(actual_ave > 500.*0.9 && actual_ave < 500.*1.1, "average of {} rolls too far from the expected mean", reps);
}

#[test]
fn fate_dice() {
	let mut dice_bag = dicexp::new_dice_bag_from_seed(12345);
	let mut sum = 0;
	let reps = 1000;
	for _ in 0..reps {
		let result = dice_bag.eval("4dF").unwrap();
		assert_eq!(result.average, 0., "wrong average");
		assert_eq!(result.min, -4, "wrong min");
		assert_eq!(result.max, 4, "wrong max");
		assert!(result.total >= result.min && result.total <= result.max, "total out of expected range");
		sum += result.total;
	}
	let mut actual_ave = sum as f64 / reps as f64;
	assert!(actual_ave > -1. && actual_ave < 1., "average of {} rolls too far from the expected mean", reps);
}

#[test]
fn exploding_dice() {
	use std::collections::HashMap;
	let mut dice_bag = dicexp::new_dice_bag_from_seed(12345);
	let max_explodes_by_die_type = HashMap::<i64, [u32;3]>::from([
		// num explosions is array for 1, 2, and 3 exploding sides
		(2,[7,0,0]),
		(4,[4,7,17]),
		(6,[3,5,7]),
		(8,[3,4,5]),
		(10,[2,3,4]),
		(12,[2,3,4]),
	]);
	for n in 1..=3 {
		for half_d in 1..=6 {
			let d = 2*half_d;
			let exp = format!("{}d{}!", n, d);
			let max_explosions = max_explodes_by_die_type[&d][0];
			let expected = dicexp::DiceRoll{
				total: 0, min: n, max: n * d * max_explosions as i64,
				average: montecarlo_exploding_dice_average(n as u32, d as u32, &[d as u32], max_explosions, 12345, 10000, 10)
			};
			let result = dice_bag.eval(&exp).unwrap();
			assert_eq!(result.min, expected.min);
			assert_eq!(result.max, expected.max);
			assert_close(result.average, expected.average, 0.05);
			if d >= 6 {
				// test multi-explode stats
				for num_x_sides in 1..=3 {
					let mut x_nums: Vec<u32> =  Vec::new();
					for x in (d-num_x_sides+1)..=d {
						x_nums.push(x as u32);
					}
					let max_explosions = max_explodes_by_die_type[&d][(num_x_sides -1) as usize];
					let expected = dicexp::DiceRoll{
						total: 0, min: n, max: n * d * max_explosions as i64,
						average: montecarlo_exploding_dice_average(n as u32, d as u32, x_nums.as_slice(), max_explosions, 12345, 10000, 10)
					};
					let exp = format!(
						"{}d{}![{}]", n, d, x_nums.iter().map(|n| n.to_string()).collect::<Vec<String>>().join(",")
					);
					let result = dice_bag.eval(&exp).unwrap();
					assert_eq!(result.min, expected.min);
					assert_eq!(result.max, expected.max);
					assert_close(result.average, expected.average, 0.05);
				}
			}
		}
	}
}

#[test]
fn explode_n_dice() {
	let mut dice_bag = dicexp::new_dice_bag_from_seed(12345);
	for n in 1..=3 {
		for half_d in 1..=6 {
			for max_explosions in 0..=2{
				let d = 2*half_d;
				let exp = format!("{}d{}!{}", n, d, max_explosions);
				let expected = dicexp::DiceRoll{
					total: 0, min: n, max: n * d * max_explosions as i64,
					average: montecarlo_exploding_dice_average(n as u32, d as u32, &[d as u32], max_explosions, 12345, 10000, 10)
				};
				let result = dice_bag.eval(&exp).unwrap();
				assert_eq!(result.min, expected.min);
				assert_eq!(result.max, expected.max);
				assert_close(result.average, expected.average, 0.05);
				if d >= 6 {
					// test multi-explode stats
					for num_x_sides in 1..=3 {
						let mut x_nums: Vec<u32> = Vec::new();
						for x in (d - num_x_sides + 1)..=d {
							x_nums.push(x as u32);
						}
						let expected = dicexp::DiceRoll {
							total: 0,
							min: n,
							max: n * d * max_explosions as i64,
							average: montecarlo_exploding_dice_average(n as u32, d as u32, x_nums.as_slice(), max_explosions, 12345, 10000, 10)
						};
						let exp = format!(
							"{}d{}!{}[{}]", n, d, max_explosions, x_nums.iter().map(|n| n.to_string()).collect::<Vec<String>>().join(",")
						);
						let result = dice_bag.eval(&exp).unwrap();
						assert_eq!(result.min, expected.min);
						assert_eq!(result.max, expected.max);
						assert_close(result.average, expected.average, 0.05);
					}
				}
			}
		}
	}
}

#[test]
fn test_shadowrun() {
	
}



/// run with `cargo test test_montecarlo_exploding_dice_average -- --no-capture` to see output
#[test]
fn test_montecarlo_exploding_dice_average(){
	for n in 1..=3 {
		for half_d in 1..=6 {
			let d = 2*half_d;
			let non_explode_ave = montecarlo_exploding_dice_average(n, d, &[d], 0, 12345, 10000, 10);
			print!("{}d{} average: {}", n, d, non_explode_ave);
			assert_close(non_explode_ave, n as f64 * (d as f64 + 1.) / 2., 0.05);
			let exploding_ave = montecarlo_exploding_dice_average(n, d, &[d], 999, 12345, 10000, 10);
			print!("\t{}d{}! average: {}", n, d, exploding_ave);
			assert!(exploding_ave > non_explode_ave);
			if d >= 4 {
				assert!(exploding_ave < non_explode_ave + n as f64);
			}
			if d >= 6 {
				// multi-explodes
				for num_x_sides in 2..=3 {
					let mut x_nums: Vec<u32> =  Vec::new();
					for x in (d-num_x_sides+1)..=d {
						x_nums.push(x);
					}
					let exploding_ave = montecarlo_exploding_dice_average(n, d, x_nums.as_slice(), 999, 12345, 10000, 10);
					print!("\t{}d{}!{:?} average: {}", n, d, x_nums, exploding_ave);
				}
			}
			println!()
		}
		println!()
	}

}

/// Simulate a large number of dice rolls and return the average (returns an average of averages)
fn montecarlo_exploding_dice_average(n: u32, d: u32, x: &[u32], max_explodes: u32, seed: u64, num_iters: u32, num_sims: u32) -> f64 {
	use rand::rngs::StdRng;
	use rand::SeedableRng;
	use std::collections::HashSet;
	let mut seed_rng = StdRng::seed_from_u64(seed);
	let mut sum_of_sims = 0.;
	let explode_set: HashSet<u32> = HashSet::from_iter(x.iter().cloned());
	assert!(d > 1);
	assert!(explode_set.len() < d as usize);
	// loop simulations
	for _ in 0..num_sims {
		let mut rng = StdRng::seed_from_u64(seed_rng.next_u64());
		let mut sum = 0.;
		// loop num repetitions in the simulation
		for _ in 0..num_iters {
			// loop n dice
			for _ in 0..n {
				let mut explosions = 0;
				let mut roll;
				loop {
					roll = rng.next_u32() % d + 1;
					sum += roll as f64;
					if explosions >= max_explodes || !explode_set.contains(&roll) { break; }
					explosions += 1;
				}
			}
		}
		let ave = sum / num_iters as f64;
		sum_of_sims += ave;
	}
	sum_of_sims / num_sims as f64
}

fn assert_close(a: f64, b: f64, t: f64) {
	let delta = (a - b).abs();
	assert!(delta < t, "near-equality check failed: difference between {} and {} is beyond tolerance {}", a, b, t);
}

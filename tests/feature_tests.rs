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
fn exploding_dice_unlimited() {
	let mut dice_bag = dicexp::new_dice_bag_from_seed(12345);
	let exp = "1d6";
	let expected = dicexp::DiceRoll{min: 1, max: 6 * 4, average: 3.5 + (1./6.)*(6.+3.5)};
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

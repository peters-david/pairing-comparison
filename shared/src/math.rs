pub fn quantile_from_sorted(sorted: &Vec<f64>, p: f64) -> f64 {
    let length = sorted.len();
    assert!(length > 0, "Cannot calculate quartile of empty data");
    let index_middle = p * (length as f64 - 1.0);
    let index_lower = index_middle.floor() as usize;
    let index_higher = index_middle.ceil() as usize;
    if index_lower == index_higher {
        sorted[index_lower]
    } else {
        let weight = index_middle - index_lower as f64;
        sorted[index_lower] * (1.0 - weight) + sorted[index_higher] * weight
    }
}

pub fn transpose<T: Clone>(input: Vec<Vec<T>>) -> Vec<Vec<T>> {
    let length = input
        .first()
        .expect("Cannot do multi zip on empty data")
        .len();
    let mut output = vec![Vec::new(); length];
    for inner in input {
        for (i, value) in inner.into_iter().enumerate() {
            output
                .get_mut(i)
                .expect("Could not get multi zip entry")
                .push(value);
        }
    }
    output
}

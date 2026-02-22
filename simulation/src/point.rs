use rand::rngs::StdRng;
use rand::RngExt;

#[derive(Debug, Clone)]
pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    pub fn random_01(rng: &mut StdRng) -> Self {
        let x = rng.random_range(0.0..=1.0);
        let y = rng.random_range(0.0..=1.0);
        Self { x, y }
    }

    pub fn distance(p1: &Self, p2: &Self) -> f64 {
        let x_delta = p1.x - p2.x;
        let y_delta = p1.y - p2.y;
        (x_delta * x_delta + y_delta * y_delta).sqrt()
    }
}


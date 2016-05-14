use rand::{self, Rand, Rng};

use math::GeoPoint;

pub struct Star {
    pub position: GeoPoint<f32>,
}

impl Rand for Star {
    fn rand<R: Rng>(rng: &mut R) -> Self {
        Star { position: rng.gen() }
    }
}

pub struct StarField {
    pub stars0: Vec<Star>,
    pub stars1: Vec<Star>,
    pub stars2: Vec<Star>,
}

impl StarField {
    pub fn generate() -> StarField {
        const STARS0_COUNT: usize = 100000;
        const STARS1_COUNT: usize = 10000;
        const STARS2_COUNT: usize = 1000;

        let mut rng = rand::weak_rng();
        StarField {
            stars0: (0..STARS0_COUNT).map(|_| rng.gen()).collect(),
            stars1: (0..STARS1_COUNT).map(|_| rng.gen()).collect(),
            stars2: (0..STARS2_COUNT).map(|_| rng.gen()).collect(),
        }
    }
}

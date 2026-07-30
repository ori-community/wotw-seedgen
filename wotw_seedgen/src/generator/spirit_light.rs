use super::SEED_FAILED_MESSAGE;
use rand::{distributions::Uniform, prelude::Distribution, SeedableRng};
use rand_pcg::Pcg64Mcg;

const MIN_SPIRIT_LIGHT: f32 = 50.;
const NOISE: f32 = 0.25;

/// We want spirit_light(i) = ai² + b (quadratic growth)
/// And b = `MIN_SPIRIT_LIGHT` (start above zero) with some random noise
/// And ₀∫ᵖ spirit_light(i) di = t (place the total amount)
///
/// SPIRIT_LIGHT(i) = ai³/3 + bi
/// ... ₀∫ᵖ spirit_light(i) di = SPIRIT_LIGHT(p) - SPIRIT_LIGHT(0) = t
/// ... ap³/3 + bp = t
/// ... a = 3(t - bp)/p³
pub struct SpiritLightProvider {
    rng: Pcg64Mcg,
    a: f32,
    i: f32,
    b: Uniform<f32>,
}

impl SpiritLightProvider {
    pub fn new(rng: &mut Pcg64Mcg) -> Self {
        Self {
            rng: Pcg64Mcg::from_rng(rng).expect(SEED_FAILED_MESSAGE),
            a: 0.,
            i: 0.,
            b: Uniform::new_inclusive(
                MIN_SPIRIT_LIGHT * (1. - NOISE),
                MIN_SPIRIT_LIGHT * (1. + NOISE),
            ),
        }
    }

    pub fn init(&mut self, total_spirit_light: i32, total_placements: usize) {
        let t = total_spirit_light as f32;
        let p = total_placements as f32;
        self.a = 3. * (t - MIN_SPIRIT_LIGHT * p) / (p * p * p);
    }

    pub fn take(&mut self) -> i32 {
        let amount = self.current_amount();
        self.i += 1.;
        amount
    }

    pub fn take_exceed(&mut self) -> i32 {
        self.current_amount()
    }

    fn current_amount(&mut self) -> i32 {
        (self.a * self.i * self.i + self.b.sample(&mut self.rng)).round() as i32
    }
}

use super::SEED_FAILED_MESSAGE;
use rand::{distributions::Uniform, prelude::Distribution, SeedableRng};
use rand_pcg::Pcg64Mcg;

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
    b: f32,
    b_noisy: Uniform<f32>,
    p: f32,
}

impl SpiritLightProvider {
    pub fn new(rng: &mut Pcg64Mcg) -> Self {
        Self {
            rng: Pcg64Mcg::from_rng(rng).expect(SEED_FAILED_MESSAGE),
            a: 0.,
            i: 0.,
            b_noisy: Uniform::new_inclusive(0., 0.),
            b: 0.,
            p: 0.,
        }
    }

    pub fn init(&mut self, total_spirit_light: i32, total_placements: usize) {
        let t = total_spirit_light as f32;
        self.b = t / 400.;
        self.p = total_placements as f32;
        self.a = 3. * (t - self.b * self.p) / (self.p * self.p * self.p);
        self.b_noisy = Uniform::new_inclusive(self.b * (1. - NOISE), self.b * (1. + NOISE));
    }

    pub fn take(&mut self) -> i32 {
        let amount = self.current_amount();
        self.i += 1.;
        amount
    }

    pub fn take_exceed(&mut self) -> i32 {
        self.current_amount()
    }

    pub fn remaining(&self) -> i32 {
        // This is ᵢ∫ᵖ spirit_light(x) dx
        // ... = SPIRIT_LIGHT(p) - SPIRIT_LIGHT(i)
        // ... = ap³/3 + bp - ai³/3 - bi
        // ... = a/3(p³ - i³) + b(p - i)
        (self.a * (1. / 3.) * (self.p * self.p * self.p - self.i * self.i * self.i)
            + self.b * (self.p - self.i))
            .round() as i32
    }

    fn current_amount(&mut self) -> i32 {
        (self.a * self.i * self.i + self.b_noisy.sample(&mut self.rng)).round() as i32
    }
}

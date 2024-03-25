use super::SEED_FAILED_MESSAGE;
use rand::{distributions::Uniform, Rng, SeedableRng};
use rand_pcg::Pcg64Mcg;
use std::mem;

const MIN_SPIRIT_LIGHT: f32 = 50.;

// TODO the last couple spirit light placements are weird
pub struct SpiritLightProvider {
    rng: Pcg64Mcg,
    amount: f32,
    next_amount: f32,
    noise: Uniform<f32>,
}
impl SpiritLightProvider {
    pub fn new(amount: i32, rng: &mut Pcg64Mcg) -> Self {
        Self {
            rng: Pcg64Mcg::from_rng(rng).expect(SEED_FAILED_MESSAGE),
            amount: amount as f32,
            next_amount: MIN_SPIRIT_LIGHT,
            noise: Uniform::new_inclusive(0.75, 1.25),
        }
    }

    // TODO in a single world seed, the reported spirit_light_placements_remaining have some jumps, not sure that should be the case
    pub fn take(&mut self, spirit_light_placements_remaining: usize) -> usize {
        // For brevity, spirit_light_placements_remaining is referred to as remaining in this comment
        //
        // We want spirit_light(remaining) = a * remaining + b (linear growth)
        // And spirit_light(remaining) = self.next_amount (avoid extreme jumps)
        // And ∫₁ˢˡᵒᵗˢʳᵉᵐᵃᶦⁿᶦⁿᵍ spirit_light dx = self.amount (aim towards placing the specified total amount)
        // And next = spirit_light(remaining - 1)
        //
        // We have to reevaluate these conditions every time because in multiworld seeds it's not possible to know upfront
        // across how many placements we need to spread out the spirit light, some worlds may end up with fewer or more spirit light items
        //
        // So spirit_light(remaining) = a * remaining + b = self.next_amount
        // ... b = self.next_amount - a * remaining
        // And ∫₁ˢˡᵒᵗˢʳᵉᵐᵃᶦⁿᶦⁿᵍ spirit_light dx = 1/2 * (remaining - 1) * (a * remaining + a + 2 * b) = self.amount
        // ... a * remaining + a + 2 * b = 2 * self.amount / (remaining - 1)
        // ... a * remaining + a + 2 * self.next_amount - 2 * a * remaining = 2 * self.amount / (remaining - 1)
        // ... a * (remaining + 1 - 2 * remaining) = 2 * self.amount / (remaining - 1) - 2 * self.next_amount
        // ... a = (2 * self.amount / (remaining - 1) - 2 * self.next_amount) / (remaining + 1 - 2 * remaining)

        let remaining = spirit_light_placements_remaining as f32;
        let a = (2. * self.amount / (remaining - 1.) - 2. * self.next_amount)
            / (remaining + 1. - 2. * remaining);
        let b = self.next_amount - a * remaining;
        let next = (a * (remaining - 1.) + b) * self.rng.sample(self.noise);
        self.amount -= self.next_amount;
        mem::replace(&mut self.next_amount, next).round() as usize
    }
}

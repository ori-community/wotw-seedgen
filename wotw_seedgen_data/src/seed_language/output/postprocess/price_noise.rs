use rand::{distributions::Uniform, prelude::Distribution};
use rand_pcg::Pcg64Mcg;

use crate::{
    seed_language::{
        ast::ArithmeticOperator,
        output::{AsConstant, CommandFloat, CommandInteger, Operation},
    },
    CommonUberIdentifier, Skill,
};

pub struct PriceNoise(Uniform<f32>);

impl PriceNoise {
    pub fn new() -> Self {
        Self(Uniform::new_inclusive(0.75, 1.25))
    }

    pub fn add_noise(&self, price: &mut CommandInteger, rng: &mut Pcg64Mcg) {
        const BLAZE_PRICE: i32 = CommonUberIdentifier::Skill(Skill::Blaze).shop_price();
        if matches!(price, CommandInteger::Constant { value: BLAZE_PRICE }) {
            return;
        }

        let factor = self.0.sample(rng);

        *price = match price.as_constant() {
            None => CommandInteger::FromFloat {
                float: Box::new(CommandFloat::from(Operation {
                    left: CommandFloat::FromInteger {
                        integer: Box::new(price.clone()),
                    },
                    operator: ArithmeticOperator::Multiply,
                    right: factor.into(),
                })),
            },
            Some(price) => ((price as f32 * factor).round() as i32).into(),
        };
    }
}

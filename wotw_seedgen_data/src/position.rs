use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use utoipa::ToSchema;

/// `x`/`y` position
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, ToSchema,
)]
pub struct Position {
    #[schema(value_type = f32)]
    pub x: OrderedFloat<f32>,
    #[schema(value_type = f32)]
    pub y: OrderedFloat<f32>,
}

impl Position {
    pub fn new<F: Into<OrderedFloat<f32>>>(x: F, y: F) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

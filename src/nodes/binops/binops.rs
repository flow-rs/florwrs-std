use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BinOpState<I1, I2> {
    I1(I1),
    I2(I2),
    None,
}

use std::hash::Hasher;

use float_ord::FloatOrd;
use serde::Serialize;

#[derive(DieselNewType, Debug, Serialize)]
pub struct DieselF32(pub f32);

impl std::hash::Hash for DieselF32 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        FloatOrd(self.0).hash(state)
    }
}

impl std::cmp::Eq for DieselF32 {}

impl std::cmp::PartialEq for DieselF32 {
    fn eq(&self, other: &Self) -> bool {
        FloatOrd(self.0).eq(&FloatOrd(other.0))
    }
}

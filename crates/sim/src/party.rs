use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Trait {
    Hardy,
    Sickly,
    Cheerful,
    Grumbler,
    Sharpshooter,
    Herbalist,
    Devout,
    Restless,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Skills {
    pub hunting: u8,
    pub medicine: u8,
    pub repair: u8,
    pub animals: u8,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Relationships {
    pub affinity: BTreeMap<String, i16>,
}
pub fn grief(affinity: i16) -> i16 {
    -10 - affinity.max(0) / 4
}

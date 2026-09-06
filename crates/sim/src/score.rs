use crate::health::PartyMember;

pub fn calculate(
    members: &[PartyMember],
    cash_cents: i64,
    inventory: &[(String, u32)],
    multiplier: f32,
) -> u32 {
    let people: u32 = members
        .iter()
        .filter(|m| m.alive)
        .map(|m| match m.health {
            76.. => 500,
            51..=75 => 400,
            26..=50 => 300,
            _ => 200,
        })
        .sum();
    let goods: u32 = inventory
        .iter()
        .map(|(id, qty)| match id.as_str() {
            "oxen" => qty * 4,
            "food" => qty / 25,
            "clothing" => qty * 2,
            _ => *qty * 2,
        })
        .sum();
    ((people + goods + (cash_cents.max(0) as u32 / 500)) as f32 * multiplier).round() as u32
}

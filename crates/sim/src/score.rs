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
    let goods: u64 = inventory
        .iter()
        .map(|(id, qty)| match id.as_str() {
            "oxen" => u64::from(*qty) * 8, // Each purchased yoke contains two oxen.
            "food" => u64::from(*qty) / 25,
            "clothing" | "wheel" | "axle" | "tongue" => u64::from(*qty) * 2,
            "ammunition" => u64::from(*qty) * 20 / 50,
            _ => 0,
        })
        .sum();
    let points = u64::from(people) + 50 + goods + cash_cents.max(0) as u64 / 500;
    (points as f64 * f64::from(multiplier)).round().clamp(0.0, u32::MAX as f64) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn counts_yokes_bullets_and_only_scoring_goods() {
        let people = vec![PartyMember::new("Ada".into())];
        let goods = [
            ("oxen", 3),
            ("ammunition", 5),
            ("food", 100),
            ("wheel", 2),
            ("clothing", 5),
            ("medicine", 5),
        ]
        .map(|(id, qty)| (id.to_owned(), qty));
        assert_eq!(calculate(&people, 1000, &goods, 2.), (500 + 50 + 24 + 2 + 4 + 4 + 10 + 2) * 2);
    }
    #[test]
    fn dead_members_do_not_score_and_large_cash_does_not_wrap() {
        let mut member = PartyMember::new("Ada".into());
        member.alive = false;
        assert_eq!(calculate(&[member], 0, &[], 1.), 50);
        assert_eq!(calculate(&[], i64::MAX, &[], 3.), u32::MAX);
    }
}

//! Deterministic trail graves for map and crossing scenes.

use crate::persist::{History, RunRecord};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grave {
    pub leader: String,
    pub mile: u32,
    pub date: String,
    pub cause: String,
    pub epitaph: String,
    pub local: bool,
}

/// Returns seeded fictional graves plus failed local journeys visible on this trail segment.
pub fn graves(history: &History, seed: u64, trail: &str, era: u16, max_miles: u32) -> Vec<Grave> {
    if max_miles == 0 {
        return Vec::new();
    }

    let mut result = seeded_graves(seed, trail, era, max_miles);
    result.extend(history.runs.iter().filter_map(|run| local_grave(run, trail, max_miles)));
    result.sort_by(|left, right| {
        left.mile
            .cmp(&right.mile)
            .then_with(|| left.leader.cmp(&right.leader))
            .then_with(|| left.date.cmp(&right.date))
            .then_with(|| left.cause.cmp(&right.cause))
            .then_with(|| left.epitaph.cmp(&right.epitaph))
    });
    result.dedup_by(|left, right| {
        left.mile == right.mile
            && left.leader == right.leader
            && left.date == right.date
            && left.cause == right.cause
            && left.epitaph == right.epitaph
    });
    result
}

fn seeded_graves(seed: u64, trail: &str, era: u16, max_miles: u32) -> Vec<Grave> {
    const LEADERS: [&str; 5] =
        ["Elias Ward", "Martha Bell", "Jonas Pike", "Clara Reed", "Samuel Vail"];
    const CAUSES: [&str; 5] =
        ["Fever", "River accident", "Winter cold", "Wagon fall", "Exhaustion"];
    const EPITAPHS: [&str; 5] = [
        "The fire was kept until dawn.",
        "Westward, with family near.",
        "A steady hand on the trail.",
        "Remembered at every camp.",
        "The road ended, love did not.",
    ];

    let mut state = seed ^ stable_hash(trail).rotate_left(17) ^ u64::from(era).rotate_left(41);
    let count = 3 + usize::try_from(next(&mut state) % 3).unwrap();
    (0..count)
        .map(|index| {
            let leader =
                LEADERS[(usize::try_from(next(&mut state)).unwrap() + index) % LEADERS.len()];
            let cause = CAUSES[(usize::try_from(next(&mut state)).unwrap() + index) % CAUSES.len()];
            let epitaph =
                EPITAPHS[(usize::try_from(next(&mut state)).unwrap() + index) % EPITAPHS.len()];
            Grave {
                leader: leader.into(),
                mile: 1 + u32::try_from(next(&mut state) % u64::from(max_miles)).unwrap(),
                date: format!("{era} day {}", 20 + (next(&mut state) % 220)),
                cause: cause.into(),
                epitaph: epitaph.into(),
                local: false,
            }
        })
        .collect()
}

fn local_grave(run: &RunRecord, trail: &str, max_miles: u32) -> Option<Grave> {
    if run.arrived || run.survivors != 0 || run.trail != trail || run.miles > max_miles {
        return None;
    }
    let cause = clean(&run.cause, "Trail ended");
    Some(Grave {
        leader: clean(&run.leader, "Unknown traveler"),
        mile: run.miles,
        date: format!("{} day {}", run.era, run.days),
        epitaph: clean(&run.epitaph, &cause),
        cause,
        local: true,
    })
}

fn clean(value: &str, fallback: &str) -> String {
    let value =
        value.chars().filter(|character| !character.is_control()).take(80).collect::<String>();
    let value = value.trim();
    if value.is_empty() {
        fallback.into()
    } else {
        value.into()
    }
}

fn stable_hash(value: &str) -> u64 {
    value.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

fn next(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut value = *state;
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(trail: &str, survivors: usize, miles: u32, epitaph: &str) -> RunRecord {
        RunRecord {
            run_id: format!("{trail}-{miles}-{survivors}"),
            leader: "Ada Boone".into(),
            seed: 9,
            trail: trail.into(),
            era: 1848,
            occupation: "farmer".into(),
            score: 0,
            survivors,
            days: 91,
            miles,
            arrived: false,
            epitaph: epitaph.into(),
            cause: "Fever".into(),
        }
    }

    #[test]
    fn seeded_graves_are_stable_and_seed_specific() {
        let history = History::default();
        let first = graves(&history, 7, "oregon", 1848, 1_000);
        assert_eq!(first, graves(&history, 7, "oregon", 1848, 1_000));
        assert_ne!(first, graves(&history, 8, "oregon", 1848, 1_000));
        assert!((3..=5).contains(&first.len()));
        assert!(first.iter().all(|grave| !grave.local));
    }

    #[test]
    fn local_graves_filter_failed_runs_by_trail_and_distance() {
        let history = History {
            runs: vec![
                run("oregon", 0, 400, "Westward"),
                run("california", 0, 400, "Southward"),
                run("oregon", 0, 900, "Too far"),
            ],
        };
        let graves = graves(&history, 2, "oregon", 1848, 500);
        assert!(graves.iter().any(|grave| grave.local && grave.epitaph == "Westward"));
        assert!(!graves
            .iter()
            .any(|grave| grave.epitaph == "Southward" || grave.epitaph == "Too far"));
    }

    #[test]
    fn surviving_failed_runs_do_not_make_graves() {
        let history = History { runs: vec![run("oregon", 1, 400, "Still here")] };
        assert!(graves(&history, 2, "oregon", 1848, 500).iter().all(|grave| !grave.local));
    }

    #[test]
    fn changed_epitaph_surfaces_in_local_grave() {
        let mut history = History { runs: vec![run("oregon", 0, 400, "Old words")] };
        assert!(graves(&history, 2, "oregon", 1848, 500)
            .iter()
            .any(|grave| grave.epitaph == "Old words"));
        history.runs[0].epitaph = "New words".into();
        assert!(graves(&history, 2, "oregon", 1848, 500)
            .iter()
            .any(|grave| grave.epitaph == "New words"));
    }

    #[test]
    fn malformed_history_fields_are_bounded_and_printable() {
        let malformed_epitaph = "\u{0007}".repeat(100);
        let mut record = run("oregon", 0, 400, &malformed_epitaph);
        record.leader = "\nAda".repeat(100);
        record.cause = "\t".repeat(100);
        let history = History { runs: vec![record] };
        let local =
            graves(&history, 2, "oregon", 1848, 500).into_iter().find(|grave| grave.local).unwrap();
        for field in [&local.leader, &local.date, &local.cause, &local.epitaph] {
            assert!(field.len() <= 80 && !field.chars().any(char::is_control));
        }
    }
}

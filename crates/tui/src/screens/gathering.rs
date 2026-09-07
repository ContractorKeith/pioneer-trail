//! UI copy and art selection for the gathering encounter. Rules stay in `pioneer-sim`.

use pioneer_sim::GatheringActivity;

pub fn heading(activity: GatheringActivity) -> &'static str {
    match activity {
        GatheringActivity::Forage => "WOODLAND FORAGE",
        GatheringActivity::Fish => "RIVERBANK FISHING",
    }
}

pub fn scene(activity: GatheringActivity) -> &'static str {
    match activity {
        GatheringActivity::Forage => "gathering_woodland.px",
        GatheringActivity::Fish => "gathering_riverbank.px",
    }
}

pub fn availability(activity: GatheringActivity) -> &'static str {
    match activity {
        GatheringActivity::Forage => "Berries, roots, and greens may be found here.",
        GatheringActivity::Fish => "The river may yield fish while you wait.",
    }
}

pub fn result_verb(activity: GatheringActivity) -> &'static str {
    match activity {
        GatheringActivity::Forage => "Foraged",
        GatheringActivity::Fish => "Caught",
    }
}

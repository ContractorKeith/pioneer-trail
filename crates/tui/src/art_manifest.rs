//! Typed manifest validation keeps asset names and frame timing out of guesswork.
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Animation {
    pub frames: Vec<String>,
    pub fps: u16,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layers {
    pub far: Vec<String>,
    pub near: Vec<String>,
    pub actors: Vec<String>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hunting {
    pub animals: Vec<String>,
    pub crosshair: Vec<String>,
    pub fps: u16,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rafting {
    pub raft: String,
    pub rocks: Vec<String>,
    pub fps: u16,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub wagon: Animation,
    pub ox: Animation,
    pub terrain: Vec<String>,
    pub rivers: Vec<String>,
    pub forts: Vec<String>,
    pub landmarks: Vec<String>,
    pub layers: Layers,
    pub hunting: Hunting,
    pub rafting: Rafting,
    pub map: String,
    #[serde(default)]
    pub gathering: Vec<String>,
}
impl Manifest {
    pub fn parse(text: &str) -> anyhow::Result<Self> {
        let manifest: Self = ron::from_str(text)?;
        for fps in [manifest.wagon.fps, manifest.ox.fps, manifest.hunting.fps, manifest.rafting.fps]
        {
            anyhow::ensure!((1..=60).contains(&fps), "invalid animation rate {fps}");
        }
        for group in [
            &manifest.wagon.frames,
            &manifest.ox.frames,
            &manifest.terrain,
            &manifest.rivers,
            &manifest.forts,
            &manifest.landmarks,
            &manifest.layers.far,
            &manifest.layers.near,
            &manifest.layers.actors,
            &manifest.hunting.animals,
            &manifest.hunting.crosshair,
            &manifest.rafting.rocks,
            &manifest.gathering,
        ] {
            anyhow::ensure!(!group.is_empty(), "empty asset group");
            for name in group {
                Self::check_asset(name)?;
            }
        }
        Self::check_asset(&manifest.map)?;
        Self::check_asset(&manifest.rafting.raft)?;
        Ok(manifest)
    }
    fn check_asset(name: &str) -> anyhow::Result<()> {
        anyhow::ensure!(crate::art::embedded(name).is_some(), "unknown pixel asset {name}");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_references_and_animation_rates() {
        let text = pioneer_data::ART.get_file("manifest.ron").unwrap().contents_utf8().unwrap();
        let manifest = Manifest::parse(text).unwrap();
        assert_eq!(manifest.wagon.frames.len(), 4);
        assert!(Manifest::parse(&text.replace("wagon_3.px", "missing.px")).is_err());
        assert!(Manifest::parse(&text.replace("fps: 30", "fps: 0")).is_err());
    }
}

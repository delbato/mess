use mess_core::artifact::Artifact as ArtifactTrait;

pub struct Artifact;

impl ArtifactTrait for Artifact {
    fn supports_aot() -> bool {
        true
    }
}
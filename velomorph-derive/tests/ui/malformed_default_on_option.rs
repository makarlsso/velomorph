use velomorph::Morph;

pub struct Source {
    pub maybe: Option<u32>,
}

#[derive(Morph)]
#[morph(from = "Source")]
pub struct Target {
    #[morph(default)]
    pub maybe: Option<u32>,
}

fn main() {}

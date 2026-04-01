use velomorph::Morph;

pub struct Source {
    pub field: Option<u32>,
}

#[derive(Morph)]
#[morph(from = "Source")]
pub struct Target {
    #[morph(skip, with = "convert")]
    pub field: u32,
}

fn convert(value: Option<u32>) -> Result<u32, &'static str> {
    value.ok_or("missing")
}

fn main() {}

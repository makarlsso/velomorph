use velomorph::Morph;

pub struct RawInput {
    pub id: Option<u64>,
    pub payload: Option<Vec<u8>>,
}

#[derive(Morph)]
#[morph(source = "RawInput")]
pub struct Target {
    pub id: u64,
}

fn main() {}

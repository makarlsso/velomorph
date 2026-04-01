use velomorph::Morph;

pub struct RawInput {
    pub id: Option<u64>,
    pub payload: Option<Vec<u8>>,
}

#[derive(Morph)]
pub struct Target {
    #[morph(renamed = "id")]
    pub id: u64,
}

fn main() {}

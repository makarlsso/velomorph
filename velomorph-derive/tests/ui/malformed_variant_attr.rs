use velomorph::Morph;

pub enum Source {
    A { field: Option<u32> },
}

#[derive(Morph)]
#[morph(from = "Source")]
pub enum Target {
    #[morph(bad = "A")]
    A { field: u32 },
}

fn main() {}

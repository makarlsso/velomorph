use std::borrow::Cow;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;
#[cfg(feature = "janitor")]
use velomorph::Janitor;
use velomorph::{Morph, TryMorph};

// 1. Raw incoming data (e.g., decoded from a Network Buffer, Protobuf, or JSON)
pub struct SourceEvent<'a> {
    // Legacy/ugly field names coming from an external system:
    pub uuid_v4: Option<Uuid>,
    pub user_str: &'a str, // A string reference pointing to the raw buffer
    pub metadata: Option<String>, // Optional metadata field
    pub retries_raw: Option<u16>,
    pub severity_raw: u8,
    pub payload: Option<Vec<u8>>, // Heavy data (e.g., a 100MB file or image)
}

// 2. The internal, high-performance Domain Model
#[derive(Morph, Debug)]
#[morph(from = "SourceEvent")]
#[morph(validate = "validate_event")]
pub struct ProcessedEvent<'a> {
    // [STRICT] Must exist, otherwise returns MorphError::MissingField.
    // We also rename from `uuid_v4` -> `id` using the field-level attribute.
    #[morph(from = "uuid_v4")]
    pub id: Uuid,

    // [ZERO-COPY] If the string isn't modified, it points directly to the source buffer.
    // Here we rename `user_str` -> `username`.
    #[morph(from = "user_str")]
    pub username: Cow<'a, str>,

    // [PASSTHROUGH] Maintained as an Option if present in the source
    pub metadata: Option<String>,

    // [DEFAULT] Falls back to expression if source is None.
    #[morph(from = "retries_raw", default = "3")]
    pub retries: u16,

    // [TRANSFORM] Converts from compact wire format.
    #[morph(from = "severity_raw", with = "convert_severity")]
    pub severity: String,

    // [SKIP] Excluded from source mapping and set to Default::default().
    #[morph(skip)]
    pub local_cache_key: String,
}

fn convert_severity(value: u8) -> Result<String, &'static str> {
    match value {
        0 => Ok("info".to_string()),
        1 => Ok("warn".to_string()),
        2 => Ok("error".to_string()),
        _ => Err("unsupported severity"),
    }
}

fn validate_event(event: &ProcessedEvent<'_>) -> Result<(), &'static str> {
    if event.id.is_nil() {
        return Err("event id cannot be nil");
    }
    Ok(())
}

fn build_raw_packet() -> SourceEvent<'static> {
    SourceEvent {
        uuid_v4: Some(Uuid::new_v4()),
        user_str: "admin_user_01",
        metadata: Some("Region: EU-North".to_string()),
        retries_raw: None,
        severity_raw: 1,
        payload: Some(vec![0u8; 100 * 1024 * 1024]), // 100 MB of heap data
    }
}

fn print_event(event: &ProcessedEvent<'_>, duration: Duration) {
    println!("Transformation completed in: {:?}", duration);
    println!("Event ID: {}", event.id);

    match &event.username {
        Cow::Borrowed(s) => println!(
            "Memory Status: Using borrowed string '{}' (Zero allocations)",
            s
        ),
        Cow::Owned(_) => println!("Memory Status: String is owned (Fallback allocation occurred)"),
    }

    if let Some(meta) = &event.metadata {
        println!("Metadata preserved: {}", meta);
    }
}

#[cfg(feature = "janitor")]
async fn morph_with_janitor(
    janitor: &Janitor,
    label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{label}");
    println!("Explicit background offloading with Janitor::offload before morph.");

    let mut raw_packet = build_raw_packet();
    println!("Incoming packet with 100MB payload received.");

    if let Some(heavy_payload) = raw_packet.payload.take() {
        janitor.offload(heavy_payload);
        println!("Payload offloaded to janitor thread.");
    }

    let start_time = std::time::Instant::now();
    let event: ProcessedEvent = raw_packet.try_morph(janitor)?;
    let duration = start_time.elapsed();
    print_event(&event, duration);

    println!("\nNote: The main thread is now free to handle the next packet.");
    println!("Janitor thread is handling deallocation in the background.");

    sleep(Duration::from_millis(100)).await;
    println!("Cleanup is proceeding/finished without jittering the main path.");
    Ok(())
}

#[cfg(feature = "janitor")]
async fn run_with_janitor() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Mode: WITH janitor feature ---");

    println!("\n### Unbounded janitor (`Janitor::new` — default, no queue cap)");
    let unbounded = Janitor::new();
    morph_with_janitor(
        &unbounded,
        "Same as `Janitor::default()`: unbounded queue; overload can grow memory.",
    )
    .await?;

    println!("\n### Bounded janitor (`Janitor::bounded(n)` — capped pending queue)");
    let bounded = Janitor::bounded(8);
    morph_with_janitor(
        &bounded,
        "Queue holds at most 8 pending drops; if full, `offload` drops on the caller (not deferred).",
    )
    .await?;

    Ok(())
}

#[cfg(not(feature = "janitor"))]
async fn run_without_janitor() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Mode: WITHOUT janitor feature ---");
    println!("This demonstrates plain morphing without background offloading.");

    let raw_packet = build_raw_packet();
    println!("Incoming packet with 100MB payload received.");

    let start_time = std::time::Instant::now();
    let event: ProcessedEvent = raw_packet.try_morph()?;
    let duration = start_time.elapsed();
    print_event(&event, duration);

    println!("\nNote: Deallocation occurs on the current execution path.");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Velomorph Showcase Start ---");
    println!("Core morph demo: strict + zero-copy + with/default/skip/validate.");

    #[cfg(feature = "janitor")]
    run_with_janitor().await?;
    #[cfg(not(feature = "janitor"))]
    run_without_janitor().await?;

    Ok(())
}

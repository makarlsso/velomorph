use std::borrow::Cow;
use std::time::Duration;
use tokio::time::sleep;
use velomorph::{Janitor, Morph, TryMorph};

// 1. Raw incoming data (e.g., decoded from a Network Buffer, Protobuf, or JSON)
pub struct RawInput<'a> {
    pub request_id: Option<u64>,
    pub user_tag: &'a str, // A string reference pointing to the raw buffer
    pub metadata: Option<String>, // Optional metadata field
    pub payload: Option<Vec<u8>>, // Heavy data (e.g., a 100MB file or image)
}

// 2. The internal, high-performance Domain Model
#[derive(Morph, Debug)]
pub struct ProcessedEvent<'a> {
    // [STRICT] Must exist, otherwise returns MorphError::MissingField
    pub request_id: u64,

    // [ZERO-COPY] If the string isn't modified, it points directly to the source buffer
    pub user_tag: Cow<'a, str>,

    // [PASSTHROUGH] Maintained as an Option if present in the source
    pub metadata: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the Janitor. In a real app, this should live in your Server State/Arc.
    let janitor = Janitor::new();

    println!("--- Velomorph Showcase Start ---");

    // SIMULATION: An incoming packet with a 100MB payload
    let raw_packet = RawInput {
        request_id: Some(8888),
        user_tag: "admin_user_01",
        metadata: Some("Region: EU-North".to_string()),
        payload: Some(vec![0u8; 100 * 1024 * 1024]), // 100 MB of heap data
    };

    println!("Incoming packet with 100MB payload received.");

    // --- THE CRITICAL TRANSFORMATION ---
    // try_morph performs three operations simultaneously:
    // 1. Offloads the 100MB payload to the Janitor thread immediately.
    // 2. Moves 'user_tag' as a reference (zero allocations).
    // 3. Validates that 'request_id' is present (Strict mode).

    let start_time = std::time::Instant::now();

    // Execute the transformation
    let event: ProcessedEvent = raw_packet.try_morph(&janitor)?;

    let duration = start_time.elapsed();

    // --- RESULTS ---
    println!("Transformation completed in: {:?}", duration);
    println!("Event ID: {}", event.request_id);

    match &event.user_tag {
        Cow::Borrowed(s) => println!(
            "Memory Status: Using borrowed string '{}' (Zero allocations)",
            s
        ),
        Cow::Owned(_) => println!("Memory Status: String is owned (Fallback allocation occurred)"),
    }

    if let Some(meta) = event.metadata {
        println!("Metadata preserved: {}", meta);
    }

    println!("\nNote: The main thread is now free to handle the next packet.");
    println!("The Janitor thread is currently dropping 100MB in the background...");

    // Demonstrate that we aren't blocked by the heavy deallocation
    sleep(Duration::from_millis(100)).await;
    println!("Cleanup is proceeding/finished without jittering the main path.");

    Ok(())
}

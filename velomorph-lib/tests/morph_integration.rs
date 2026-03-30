use std::borrow::Cow;

use velomorph::{Janitor, Morph, MorphError, TryMorph};

pub struct RawInput<'a> {
    pub request_id: Option<u64>,
    pub user_tag: &'a str,
    pub metadata: Option<String>,
    pub payload: Option<Vec<u8>>,
}

#[derive(Morph, Debug)]
pub struct ProcessedEvent<'a> {
    pub request_id: u64,
    pub user_tag: Cow<'a, str>,
    pub metadata: Option<String>,
}

pub struct Packet<'a> {
    pub id: Option<u64>,
    pub user_tag: &'a str,
    pub payload: Option<Vec<u8>>,
}

#[derive(Morph, Debug)]
#[morph(from = "Packet")]
pub struct PacketView<'a> {
    pub id: u64,
    pub user_tag: Cow<'a, str>,
}

#[test]
fn morph_maps_fields_and_uses_borrowed_cow() {
    let janitor = Janitor::new();
    let raw = RawInput {
        request_id: Some(7),
        user_tag: "alpha",
        metadata: Some("region-a".to_string()),
        payload: Some(vec![1, 2, 3, 4]),
    };

    let mapped: ProcessedEvent = raw.try_morph(&janitor).expect("morph should succeed");

    assert_eq!(mapped.request_id, 7);
    assert_eq!(mapped.metadata.as_deref(), Some("region-a"));
    assert!(matches!(mapped.user_tag, Cow::Borrowed("alpha")));
}

#[test]
fn morph_returns_missing_field_error_for_strict_fields() {
    let janitor = Janitor::new();
    let raw = RawInput {
        request_id: None,
        user_tag: "alpha",
        metadata: None,
        payload: None,
    };

    let err = raw
        .try_morph(&janitor)
        .expect_err("missing strict field should fail");

    assert!(matches!(err, MorphError::MissingField(field) if field == "request_id"));
}

#[test]
fn morph_supports_custom_source_type_attribute() {
    let janitor = Janitor::default();
    let packet = Packet {
        id: Some(99),
        user_tag: "tag-99",
        payload: Some(vec![0; 16]),
    };

    let view: PacketView =
        TryMorph::try_morph(packet, &janitor).expect("custom source should morph");

    assert_eq!(view.id, 99);
    assert!(matches!(view.user_tag, Cow::Borrowed("tag-99")));
}

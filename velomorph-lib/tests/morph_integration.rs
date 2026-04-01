use std::borrow::Cow;

#[cfg(feature = "janitor")]
use velomorph::Janitor;
use velomorph::{Morph, MorphError, TryMorph};

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

pub struct TransformSource {
    pub count_raw: u32,
    pub threshold: Option<u8>,
    pub payload: Option<Vec<u8>>,
}

#[derive(Morph, Debug)]
#[morph(from = "TransformSource", validate = "validate_transform_target")]
pub struct TransformTarget {
    #[morph(from = "count_raw", with = "to_count")]
    pub count: u64,
    #[morph(default = "7")]
    pub threshold: u8,
    #[morph(skip)]
    pub skipped: String,
}

fn to_count(value: u32) -> Result<u64, &'static str> {
    if value > 1000 {
        return Err("count is too high");
    }
    Ok(value as u64)
}

fn validate_transform_target(value: &TransformTarget) -> Result<(), &'static str> {
    if value.count == 0 {
        return Err("count must be greater than zero");
    }
    Ok(())
}

pub struct DefaultOnlySource {
    pub amount: Option<u32>,
    pub payload: Option<Vec<u8>>,
}

#[derive(Morph, Debug)]
#[morph(from = "DefaultOnlySource")]
pub struct DefaultOnlyTarget {
    #[morph(default)]
    pub amount: u32,
}

pub enum SourceSignal {
    Up {
        code: Option<u32>,
        label: &'static str,
    },
    LegacyDown {
        code: Option<u32>,
        label: &'static str,
    },
}

#[derive(Morph, Debug)]
#[morph(from = "SourceSignal")]
pub enum TargetSignal {
    Up {
        code: u32,
        label: Cow<'static, str>,
    },
    #[morph(from = "LegacyDown")]
    Down {
        code: u32,
        label: Cow<'static, str>,
    },
}

#[test]
fn morph_maps_fields_and_uses_borrowed_cow() {
    #[cfg(feature = "janitor")]
    let janitor = Janitor::new();
    let raw = RawInput {
        request_id: Some(7),
        user_tag: "alpha",
        metadata: Some("region-a".to_string()),
        payload: Some(vec![1, 2, 3, 4]),
    };

    #[cfg(feature = "janitor")]
    let mapped: ProcessedEvent = raw.try_morph(&janitor).expect("morph should succeed");
    #[cfg(not(feature = "janitor"))]
    let mapped: ProcessedEvent = raw.try_morph().expect("morph should succeed");

    assert_eq!(mapped.request_id, 7);
    assert_eq!(mapped.metadata.as_deref(), Some("region-a"));
    assert!(matches!(mapped.user_tag, Cow::Borrowed("alpha")));
}

#[test]
fn morph_returns_missing_field_error_for_strict_fields() {
    #[cfg(feature = "janitor")]
    let janitor = Janitor::new();
    let raw = RawInput {
        request_id: None,
        user_tag: "alpha",
        metadata: None,
        payload: None,
    };

    #[cfg(feature = "janitor")]
    let err = raw
        .try_morph(&janitor)
        .expect_err("missing strict field should fail");
    #[cfg(not(feature = "janitor"))]
    let err = raw
        .try_morph()
        .expect_err("missing strict field should fail");

    assert!(matches!(err, MorphError::MissingField(field) if field == "request_id"));
}

#[test]
fn morph_supports_custom_source_type_attribute() {
    #[cfg(feature = "janitor")]
    let janitor = Janitor::default();
    let packet = Packet {
        id: Some(99),
        user_tag: "tag-99",
        payload: Some(vec![0; 16]),
    };

    #[cfg(feature = "janitor")]
    let view: PacketView =
        TryMorph::try_morph(packet, &janitor).expect("custom source should morph");
    #[cfg(not(feature = "janitor"))]
    let view: PacketView = TryMorph::try_morph(packet).expect("custom source should morph");

    assert_eq!(view.id, 99);
    assert!(matches!(view.user_tag, Cow::Borrowed("tag-99")));
}

#[test]
fn morph_supports_with_default_skip_and_validate() {
    #[cfg(feature = "janitor")]
    let janitor = Janitor::default();
    let src = TransformSource {
        count_raw: 22,
        threshold: None,
        payload: None,
    };

    #[cfg(feature = "janitor")]
    let out: TransformTarget = TryMorph::try_morph(src, &janitor).expect("morph should succeed");
    #[cfg(not(feature = "janitor"))]
    let out: TransformTarget = TryMorph::try_morph(src).expect("morph should succeed");

    assert_eq!(out.count, 22);
    assert_eq!(out.threshold, 7);
    assert_eq!(out.skipped, String::default());
}

#[test]
fn morph_reports_transform_errors_from_with_attribute() {
    #[cfg(feature = "janitor")]
    let janitor = Janitor::default();
    let src = TransformSource {
        count_raw: 2001,
        threshold: Some(1),
        payload: None,
    };

    #[cfg(feature = "janitor")]
    let err = TryMorph::<TransformTarget>::try_morph(src, &janitor).expect_err("should fail");
    #[cfg(not(feature = "janitor"))]
    let err = TryMorph::<TransformTarget>::try_morph(src).expect_err("should fail");

    assert!(matches!(err, MorphError::TransformError(msg) if msg.contains("too high")));
}

#[test]
fn morph_reports_validation_errors_from_validate_attribute() {
    #[cfg(feature = "janitor")]
    let janitor = Janitor::default();
    let src = TransformSource {
        count_raw: 0,
        threshold: Some(1),
        payload: None,
    };

    #[cfg(feature = "janitor")]
    let err = TryMorph::<TransformTarget>::try_morph(src, &janitor).expect_err("should fail");
    #[cfg(not(feature = "janitor"))]
    let err = TryMorph::<TransformTarget>::try_morph(src).expect_err("should fail");

    assert!(matches!(err, MorphError::ValidationError(msg) if msg.contains("greater than zero")));
}

#[test]
fn morph_supports_default_trait_attribute() {
    #[cfg(feature = "janitor")]
    let janitor = Janitor::default();
    let src = DefaultOnlySource {
        amount: None,
        payload: None,
    };

    #[cfg(feature = "janitor")]
    let out: DefaultOnlyTarget = TryMorph::try_morph(src, &janitor).expect("morph should succeed");
    #[cfg(not(feature = "janitor"))]
    let out: DefaultOnlyTarget = TryMorph::try_morph(src).expect("morph should succeed");

    assert_eq!(out.amount, 0);
}

#[test]
fn morph_supports_enum_same_name_and_variant_override() {
    #[cfg(feature = "janitor")]
    let janitor = Janitor::default();
    let up = SourceSignal::Up {
        code: Some(10),
        label: "u",
    };
    let down = SourceSignal::LegacyDown {
        code: Some(11),
        label: "d",
    };

    #[cfg(feature = "janitor")]
    let mapped_up: TargetSignal = TryMorph::try_morph(up, &janitor).expect("up should morph");
    #[cfg(not(feature = "janitor"))]
    let mapped_up: TargetSignal = TryMorph::try_morph(up).expect("up should morph");
    #[cfg(feature = "janitor")]
    let mapped_down: TargetSignal = TryMorph::try_morph(down, &janitor).expect("down should morph");
    #[cfg(not(feature = "janitor"))]
    let mapped_down: TargetSignal = TryMorph::try_morph(down).expect("down should morph");

    assert!(matches!(
        mapped_up,
        TargetSignal::Up {
            code: 10,
            label: Cow::Borrowed("u")
        }
    ));
    assert!(matches!(
        mapped_down,
        TargetSignal::Down {
            code: 11,
            label: Cow::Borrowed("d")
        }
    ));
}

#[test]
fn morph_supports_vec_to_vec_mapping() {
    #[cfg(feature = "janitor")]
    let janitor = Janitor::default();
    let input = vec![
        TransformSource {
            count_raw: 10,
            threshold: Some(3),
            payload: None,
        },
        TransformSource {
            count_raw: 20,
            threshold: None,
            payload: Some(vec![1, 2, 3]),
        },
    ];

    #[cfg(feature = "janitor")]
    let out: Vec<TransformTarget> =
        TryMorph::try_morph(input, &janitor).expect("vec morph succeeds");
    #[cfg(not(feature = "janitor"))]
    let out: Vec<TransformTarget> = TryMorph::try_morph(input).expect("vec morph succeeds");

    assert_eq!(out.len(), 2);
    assert_eq!(out[0].count, 10);
    assert_eq!(out[0].threshold, 3);
    assert_eq!(out[1].count, 20);
    assert_eq!(out[1].threshold, 7);
}

#[test]
fn morph_vec_to_vec_propagates_first_error() {
    #[cfg(feature = "janitor")]
    let janitor = Janitor::default();
    let input = vec![
        TransformSource {
            count_raw: 10,
            threshold: Some(1),
            payload: None,
        },
        TransformSource {
            count_raw: 2001,
            threshold: Some(1),
            payload: None,
        },
        TransformSource {
            count_raw: 15,
            threshold: Some(1),
            payload: None,
        },
    ];

    #[cfg(feature = "janitor")]
    let err =
        TryMorph::<Vec<TransformTarget>>::try_morph(input, &janitor).expect_err("vec morph fails");
    #[cfg(not(feature = "janitor"))]
    let err = TryMorph::<Vec<TransformTarget>>::try_morph(input).expect_err("vec morph fails");

    assert!(matches!(err, MorphError::TransformError(msg) if msg.contains("too high")));
}

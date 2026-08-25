use std::str::FromStr;

use lawsynth_quant::{Currency, Money, ObservationKey, QuantError, UtcTimestamp};

#[test]
fn currency_registry_is_closed_and_declares_minor_units() {
    assert_eq!(Currency::from_str("USD").unwrap(), Currency::Usd);
    assert_eq!(Currency::Usd.minor_unit_exponent(), 2);
    assert_eq!(Currency::Jpy.minor_unit_exponent(), 0);
    assert!(matches!(Currency::from_str("usd"), Err(QuantError::UnknownCurrency(_))));
    assert!(matches!(Currency::from_str("XYZ"), Err(QuantError::UnknownCurrency(_))));
}

#[test]
fn money_is_exact_currency_safe_and_overflow_checked() {
    let first = Money::from_minor_units(Currency::Usd, 125);
    let second = Money::from_minor_units(Currency::Usd, -25);
    assert_eq!(first.checked_add(second).unwrap().minor_units(), 100);
    assert_eq!(first.checked_sub(second).unwrap().minor_units(), 150);

    let eur = Money::from_minor_units(Currency::Eur, 125);
    assert!(matches!(first.checked_add(eur), Err(QuantError::CurrencyMismatch { .. })));
    let maximum = Money::from_minor_units(Currency::Usd, i128::MAX);
    assert_eq!(
        maximum.checked_add(Money::from_minor_units(Currency::Usd, 1)),
        Err(QuantError::ArithmeticOverflow)
    );
}

#[test]
fn money_exact_integer_algebra_preserves_currency_and_checks_overflow() {
    let unit = Money::from_minor_units(Currency::Usd, 250);
    let scaled = unit.checked_mul(4).unwrap();
    assert_eq!(scaled.minor_units(), 1_000);
    assert_eq!(scaled.currency(), Currency::Usd);
    assert_eq!(unit.checked_mul(0).unwrap().minor_units(), 0);

    let debit = Money::from_minor_units(Currency::Try, -75);
    assert_eq!(debit.checked_neg().unwrap().minor_units(), 75);
    assert_eq!(debit.checked_abs().unwrap().minor_units(), 75);
    assert_eq!(debit.checked_neg().unwrap().currency(), Currency::Try);
    assert!(Money::from_minor_units(Currency::Usd, 0).is_zero());
    assert!(!debit.is_zero());

    let maximum = Money::from_minor_units(Currency::Usd, i128::MAX);
    assert_eq!(maximum.checked_mul(2), Err(QuantError::ArithmeticOverflow));
    let minimum = Money::from_minor_units(Currency::Usd, i128::MIN);
    assert_eq!(minimum.checked_neg(), Err(QuantError::ArithmeticOverflow));
    assert_eq!(minimum.checked_abs(), Err(QuantError::ArithmeticOverflow));
}

#[test]
fn money_encoding_round_trips_and_rejects_drift() {
    let amount = Money::from_minor_units(Currency::Try, -12_345);
    let encoded = amount.canonical_bytes();
    assert_eq!(&encoded[..8], b"LSQM1TRY");
    assert_eq!(Money::from_canonical_bytes(&encoded).unwrap(), amount);
    assert_eq!(amount.stable_fingerprint(), amount.stable_fingerprint());

    let mut unknown_version = encoded;
    unknown_version[4] = b'2';
    assert!(matches!(
        Money::from_canonical_bytes(&unknown_version),
        Err(QuantError::InvalidEncoding(_))
    ));
    assert!(Money::from_canonical_bytes(&encoded[..23]).is_err());
}

#[test]
fn observation_identity_is_ordered_and_deterministic() {
    let timestamp = UtcTimestamp::from_unix_millis(1_787_486_400_000);
    let first = ObservationKey::new("AAPL-XNAS", timestamp, 0).unwrap();
    let second = ObservationKey::new("AAPL-XNAS", timestamp, 1).unwrap();
    assert!(first < second);
    assert_eq!(first.timestamp().unix_millis(), 1_787_486_400_000);
    assert_eq!(first.instrument().as_str(), "AAPL-XNAS");

    let encoded = first.canonical_bytes();
    assert_eq!(ObservationKey::from_canonical_bytes(&encoded).unwrap(), first);
    assert_eq!(first.stable_fingerprint(), first.stable_fingerprint());
}

#[test]
fn observation_decoder_rejects_ambiguous_or_malformed_identity() {
    assert!(matches!(
        ObservationKey::new("AAPL XNAS", UtcTimestamp::from_unix_millis(0), 0),
        Err(QuantError::InvalidInstrument(_))
    ));

    let valid = ObservationKey::new("USDTRY", UtcTimestamp::from_unix_millis(-1), 7)
        .unwrap()
        .canonical_bytes();
    let mut wrong_length = valid.clone();
    wrong_length[6] = wrong_length[6].saturating_add(1);
    assert!(matches!(
        ObservationKey::from_canonical_bytes(&wrong_length),
        Err(QuantError::InvalidEncoding(_))
    ));
    assert!(ObservationKey::from_canonical_bytes(&valid[..valid.len() - 1]).is_err());
}

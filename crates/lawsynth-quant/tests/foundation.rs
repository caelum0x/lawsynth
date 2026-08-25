use std::str::FromStr;

use lawsynth_quant::{
    Currency, Direction, Lot, Money, ObservationKey, Position, QuantError, UtcTimestamp,
};

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
fn money_decoder_rejects_corrupt_currency_field() {
    let encoded = Money::from_minor_units(Currency::Usd, 4_200).canonical_bytes();

    // Non-UTF-8 currency-code bytes are rejected before registry lookup.
    let mut not_utf8 = encoded;
    not_utf8[5..8].copy_from_slice(&[0xFF, 0xFE, 0xFD]);
    assert!(matches!(Money::from_canonical_bytes(&not_utf8), Err(QuantError::InvalidEncoding(_))));

    // Valid UTF-8 but an unknown code is rejected by the closed registry.
    let mut unknown_code = encoded;
    unknown_code[5..8].copy_from_slice(b"XXX");
    assert!(matches!(
        Money::from_canonical_bytes(&unknown_code),
        Err(QuantError::UnknownCurrency(_))
    ));
}

#[test]
fn position_values_holdings_through_exact_money_algebra() {
    let price = Money::from_minor_units(Currency::Usd, 15_000);
    let long = Position::new("AAPL-XNAS", 3).unwrap();
    assert_eq!(long.direction(), Direction::Long);
    assert!(!long.is_flat());
    // market_value = price * quantity (checked_mul).
    assert_eq!(long.market_value(price).unwrap().minor_units(), 45_000);
    // notional = |market_value| (checked_abs).
    assert_eq!(long.notional(price).unwrap().minor_units(), 45_000);
    // establishing a long is a cash outflow: -(price * quantity) (checked_neg).
    let outflow = long.establish_cash_flow(price).unwrap();
    assert_eq!(outflow.minor_units(), -45_000);
    assert_eq!(outflow.currency(), Currency::Usd);

    let short = Position::new("AAPL-XNAS", -2).unwrap();
    assert_eq!(short.direction(), Direction::Short);
    assert_eq!(short.market_value(price).unwrap().minor_units(), -30_000);
    // notional ignores sign; shorting raises cash.
    assert_eq!(short.notional(price).unwrap().minor_units(), 30_000);
    assert_eq!(short.establish_cash_flow(price).unwrap().minor_units(), 30_000);

    let flat = Position::new("AAPL-XNAS", 0).unwrap();
    assert_eq!(flat.direction(), Direction::Flat);
    assert!(flat.is_flat());
    assert!(flat.market_value(price).unwrap().is_zero());
}

#[test]
fn position_netting_reverse_and_overflow_are_checked() {
    let long = Position::new("USDTRY", 5).unwrap();
    let short = Position::new("USDTRY", -8).unwrap();
    assert_eq!(long.combine(&short).unwrap().quantity(), -3);
    assert_eq!(long.reverse().unwrap().quantity(), -5);
    assert_eq!(long.combine(&long.reverse().unwrap()).unwrap().direction(), Direction::Flat);

    let other = Position::new("AAPL-XNAS", 1).unwrap();
    assert!(matches!(long.combine(&other), Err(QuantError::InstrumentMismatch { .. })));
    assert!(matches!(Position::new("US DTRY", 1), Err(QuantError::InvalidInstrument(_))));

    let max = Position::new("USDTRY", i64::MAX).unwrap();
    let one = Position::new("USDTRY", 1).unwrap();
    assert_eq!(max.combine(&one), Err(QuantError::ArithmeticOverflow));
    let min = Position::new("USDTRY", i64::MIN).unwrap();
    assert_eq!(min.reverse(), Err(QuantError::ArithmeticOverflow));

    // Money-scale overflow surfaces through valuation, not wrapping.
    let huge = Money::from_minor_units(Currency::Usd, i128::MAX);
    assert_eq!(
        Position::new("USDTRY", 2).unwrap().market_value(huge),
        Err(QuantError::ArithmeticOverflow)
    );
}

#[test]
fn position_encoding_round_trips_and_rejects_drift() {
    let position = Position::new("USDTRY", -12_345).unwrap();
    let encoded = position.canonical_bytes();
    assert_eq!(&encoded[..5], b"LSQP1");
    assert_eq!(Position::from_canonical_bytes(&encoded).unwrap(), position);
    assert_eq!(position.stable_fingerprint(), position.stable_fingerprint());

    let mut unknown_version = encoded.clone();
    unknown_version[4] = b'2';
    assert!(matches!(
        Position::from_canonical_bytes(&unknown_version),
        Err(QuantError::InvalidEncoding(_))
    ));
    let mut wrong_length = encoded.clone();
    wrong_length[6] = wrong_length[6].saturating_add(1);
    assert!(matches!(
        Position::from_canonical_bytes(&wrong_length),
        Err(QuantError::InvalidEncoding(_))
    ));
    assert!(Position::from_canonical_bytes(&encoded[..encoded.len() - 1]).is_err());
}

#[test]
fn lot_marks_profit_and_loss_through_exact_money_algebra() {
    let entry = Money::from_minor_units(Currency::Usd, 15_000);
    let long = Lot::new(Position::new("AAPL-XNAS", 3).unwrap(), entry);
    assert_eq!(long.entry_price(), entry);
    assert_eq!(long.position().direction(), Direction::Long);
    // cost basis = entry * quantity (checked_mul).
    assert_eq!(long.entry_value().unwrap().minor_units(), 45_000);

    let mark_up = Money::from_minor_units(Currency::Usd, 16_000);
    // market_value = mark * quantity; unrealized = quantity * (mark - entry).
    assert_eq!(long.market_value(mark_up).unwrap().minor_units(), 48_000);
    assert_eq!(long.unrealized_pnl(mark_up).unwrap().minor_units(), 3_000);
    // A long loses when the mark falls below entry.
    let mark_down = Money::from_minor_units(Currency::Usd, 14_500);
    assert_eq!(long.unrealized_pnl(mark_down).unwrap().minor_units(), -1_500);
    // At the entry price P&L is exactly zero.
    assert!(long.unrealized_pnl(entry).unwrap().is_zero());

    // A short profits when the mark falls: -2 * (14_500 - 15_000) = +1_000.
    let short = Lot::new(Position::new("AAPL-XNAS", -2).unwrap(), entry);
    assert_eq!(short.unrealized_pnl(mark_down).unwrap().minor_units(), 1_000);
    assert_eq!(short.unrealized_pnl(mark_up).unwrap().minor_units(), -2_000);
    assert_eq!(short.entry_value().unwrap().minor_units(), -30_000);
}

#[test]
fn lot_pnl_rejects_currency_mismatch_and_overflow() {
    let entry = Money::from_minor_units(Currency::Usd, 100);
    let lot = Lot::new(Position::new("AAPL-XNAS", 1).unwrap(), entry);
    // A differing mark currency is rejected, never silently converted.
    let eur_mark = Money::from_minor_units(Currency::Eur, 100);
    assert!(matches!(lot.unrealized_pnl(eur_mark), Err(QuantError::CurrencyMismatch { .. })));

    // Overflow surfaces from the per-unit price move rather than wrapping.
    let min_entry =
        Lot::new(Position::new("AAPL-XNAS", 1).unwrap(), Money::from_minor_units(Currency::Usd, 1));
    let huge_mark = Money::from_minor_units(Currency::Usd, i128::MIN);
    assert_eq!(min_entry.unrealized_pnl(huge_mark), Err(QuantError::ArithmeticOverflow));

    // Overflow also surfaces from scaling the price move by the quantity.
    let scaled =
        Lot::new(Position::new("AAPL-XNAS", 2).unwrap(), Money::from_minor_units(Currency::Usd, 0));
    let big_mark = Money::from_minor_units(Currency::Usd, i128::MAX);
    assert_eq!(scaled.unrealized_pnl(big_mark), Err(QuantError::ArithmeticOverflow));
}

#[test]
fn lot_encoding_round_trips_and_rejects_drift() {
    let lot = Lot::new(
        Position::new("USDTRY", -12_345).unwrap(),
        Money::from_minor_units(Currency::Try, 6_789),
    );
    let encoded = lot.canonical_bytes();
    assert_eq!(&encoded[..5], b"LSQL1");
    // Header carries the entry price (money segment) before the position.
    assert_eq!(&encoded[5..13], b"LSQM1TRY");
    assert_eq!(Lot::from_canonical_bytes(&encoded).unwrap(), lot);
    assert_eq!(lot.stable_fingerprint(), lot.stable_fingerprint());

    let mut unknown_version = encoded.clone();
    unknown_version[4] = b'2';
    assert!(matches!(
        Lot::from_canonical_bytes(&unknown_version),
        Err(QuantError::InvalidEncoding(_))
    ));
    // A corrupt embedded position length is rejected, not truncated silently.
    let mut wrong_length = encoded.clone();
    let position_len_hi = 5 + 24 + 5;
    wrong_length[position_len_hi + 1] = wrong_length[position_len_hi + 1].saturating_add(1);
    assert!(matches!(
        Lot::from_canonical_bytes(&wrong_length),
        Err(QuantError::InvalidEncoding(_))
    ));
    assert!(Lot::from_canonical_bytes(&encoded[..encoded.len() - 1]).is_err());
    assert!(Lot::from_canonical_bytes(&encoded[..10]).is_err());
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

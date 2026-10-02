extern crate std;
use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::{
        storage::Persistent as _, Address as _, Events as _, Ledger as _, MockAuth, MockAuthInvoke,
    },
    xdr::{Limits, WriteXdr},
    Address, Env, IntoVal, String, Symbol,
};
use std::println;

fn setup() -> (
    Env,
    Address,
    TicketReservationClient<'static>,
    Address,
    Address,
) {
    let env = Env::default();
    let organizer = Address::generate(&env);
    let attendee = Address::generate(&env);
    env.mock_all_auths();
    let id = env.register(TicketReservation, (&organizer,));
    let client = TicketReservationClient::new(&env, &id);
    (env, id, client, organizer, attendee)
}

fn title(env: &Env) -> String {
    String::from_str(env, "SoroPulse Demo")
}

#[test]
fn constructor_and_organizer_are_fixed() {
    let (env, _id, client, organizer, _attendee) = setup();
    assert_eq!(client.get_organizer(), organizer);
    assert_eq!(client.schema_version(), SCHEMA_VERSION);
    let other = Address::generate(&env);
    assert_eq!(
        client.try_create_event(&other, &1, &title(&env), &2),
        Err(Ok(Error::Unauthorized))
    );
    assert_eq!(
        client.try_close_event(&other, &1),
        Err(Ok(Error::Unauthorized))
    );
}

#[test]
fn event_validation_and_monotonic_ids() {
    let (env, _id, client, organizer, _) = setup();
    assert_eq!(
        client.try_create_event(&organizer, &1, &title(&env), &0),
        Err(Ok(Error::InvalidCapacity))
    );
    assert_eq!(
        client.try_create_event(&organizer, &1, &title(&env), &(MAX_CAPACITY + 1)),
        Err(Ok(Error::InvalidCapacity))
    );
    assert_eq!(
        client.try_create_event(&organizer, &1, &String::from_str(&env, ""), &1),
        Err(Ok(Error::InvalidTitle))
    );
    let long = String::from_str(&env, &"x".repeat((MAX_TITLE_BYTES + 1) as usize));
    assert_eq!(
        client.try_create_event(&organizer, &1, &long, &1),
        Err(Ok(Error::InvalidTitle))
    );
    let max = String::from_str(&env, &"x".repeat(MAX_TITLE_BYTES as usize));
    client.create_event(&organizer, &1, &max, &1);
    assert_eq!(client.get_event(&1).title, max);
    assert_eq!(
        client.try_create_event(&organizer, &1, &title(&env), &1),
        Err(Ok(Error::DuplicateEventId))
    );
    assert_eq!(
        client.try_create_event(&organizer, &0, &title(&env), &1),
        Err(Ok(Error::DuplicateEventId))
    );
    client.create_event(&organizer, &3, &title(&env), &1);
    assert_eq!(
        client.try_create_event(&organizer, &2, &title(&env), &1),
        Err(Ok(Error::DuplicateEventId))
    );
}

#[test]
fn reservation_capacity_duplicate_and_independence() {
    let (env, _, client, organizer, attendee) = setup();
    let other = Address::generate(&env);
    client.create_event(&organizer, &1, &title(&env), &1);
    client.create_event(&organizer, &2, &title(&env), &2);
    assert_eq!(
        client.try_reserve(&99, &attendee),
        Err(Ok(Error::EventNotFound))
    );
    let first = client.reserve(&1, &attendee);
    assert_eq!(first.id, 1);
    assert_eq!(client.get_event(&1).reserved, 1);
    assert_eq!(
        client.try_reserve(&1, &attendee),
        Err(Ok(Error::AlreadyReserved))
    );
    assert_eq!(
        client.try_reserve(&1, &other),
        Err(Ok(Error::CapacityReached))
    );
    assert_eq!(client.reserve(&2, &attendee).id, 2);
    assert_eq!(client.reserve(&2, &other).id, 3);
    assert!(client.has_reserved(&1, &attendee));
    assert!(!client.has_reserved(&1, &other));
    assert_eq!(client.get_reservation(&1, &attendee), Some(first));
    assert_eq!(client.get_reservation(&1, &other), None);
    assert_eq!(client.get_event(&2).reserved, 2);
}

#[test]
fn close_is_idempotent_and_preserves_reservations() {
    let (env, _, client, organizer, attendee) = setup();
    client.create_event(&organizer, &1, &title(&env), &2);
    let reservation = client.reserve(&1, &attendee);
    client.close_event(&organizer, &1);
    let close_events = env.events().all().events().len();
    assert_eq!(close_events, 1);
    client.close_event(&organizer, &1);
    assert_eq!(env.events().all().events().len(), 0);
    assert!(client.get_event(&1).closed);
    assert_eq!(client.get_reservation(&1, &attendee), Some(reservation));
    assert_eq!(
        client.try_reserve(&1, &Address::generate(&env)),
        Err(Ok(Error::ReservationsClosed))
    );
}

#[test]
fn failed_operations_leave_state_and_events_unchanged() {
    let (env, _, client, organizer, attendee) = setup();
    client.create_event(&organizer, &1, &title(&env), &1);
    client.reserve(&1, &attendee);
    let before_state = client.get_event(&1);
    assert_eq!(
        client.try_reserve(&1, &attendee),
        Err(Ok(Error::AlreadyReserved))
    );
    assert_eq!(
        client.try_create_event(&organizer, &1, &title(&env), &1),
        Err(Ok(Error::DuplicateEventId))
    );
    assert_eq!(client.get_event(&1), before_state);
    assert_eq!(env.events().all().events().len(), 0);
}

#[test]
fn required_auth_is_specific_to_each_address() {
    let (env, id, client, organizer, attendee) = setup();
    let event_title = title(&env);
    env.mock_auths(&[MockAuth {
        address: &organizer,
        invoke: &MockAuthInvoke {
            contract: &id,
            fn_name: "create_event",
            args: (&organizer, 1_u64, &event_title, 2_u32).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.create_event(&organizer, &1, &event_title, &2);
    env.mock_auths(&[MockAuth {
        address: &attendee,
        invoke: &MockAuthInvoke {
            contract: &id,
            fn_name: "reserve",
            args: (1_u64, &attendee).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.reserve(&1, &attendee);
    assert_eq!(client.get_event(&1).reserved, 1);
}

#[test]
fn emitted_topics_and_data_are_exact() {
    let (env, id, client, organizer, attendee) = setup();
    client.create_event(&organizer, &7, &title(&env), &2);
    let created = env.events().all();
    let expected_created = soroban_sdk::vec![
        &env,
        (
            id.clone(),
            (symbol_short!("created"), 1_u32, 7_u64).into_val(&env),
            Map::<Symbol, soroban_sdk::Val>::from_array(
                &env,
                [
                    (symbol_short!("title"), title(&env).into_val(&env)),
                    (symbol_short!("capacity"), 2_u32.into_val(&env)),
                ]
            )
            .into_val(&env)
        ),
    ];
    assert_eq!(created, expected_created);
    let reservation = client.reserve(&7, &attendee);
    let reserved = env.events().all();
    let expected_reserved = soroban_sdk::vec![
        &env,
        (
            id.clone(),
            (symbol_short!("ticket"), 1_u32, 7_u64, reservation.id).into_val(&env),
            Map::<Symbol, soroban_sdk::Val>::from_array(
                &env,
                [
                    (symbol_short!("attendee"), attendee.into_val(&env)),
                    (Symbol::new(&env, "reserved_after"), 1_u32.into_val(&env)),
                ]
            )
            .into_val(&env)
        ),
    ];
    assert_eq!(reserved, expected_reserved);
    client.close_event(&organizer, &7);
    let closed = env.events().all();
    let expected_closed = soroban_sdk::vec![
        &env,
        (
            id,
            (symbol_short!("closed"), 1_u32, 7_u64).into_val(&env),
            Map::<Symbol, soroban_sdk::Val>::from_array(
                &env,
                [(symbol_short!("reserved"), 1_u32.into_val(&env)),]
            )
            .into_val(&env)
        ),
    ];
    assert_eq!(closed, expected_closed);
}

#[test]
fn ttl_refresh_keeps_event_live() {
    let (env, id, client, organizer, attendee) = setup();
    client.create_event(&organizer, &1, &title(&env), &2);
    let old_ttl = env.as_contract(&id, || env.storage().persistent().get_ttl(&Key::Event(1)));
    env.ledger().with_mut(|li| li.sequence_number += 450_000);
    client.refresh_event(&1);
    let new_ttl = env.as_contract(&id, || env.storage().persistent().get_ttl(&Key::Event(1)));
    assert!(new_ttl > old_ttl - 450_000);
    client.reserve(&1, &attendee);
    assert_eq!(client.get_event(&1).reserved, 1);
}

#[test]
fn print_local_xdr_fixture() {
    let (env, _, client, organizer, attendee) = setup();
    client.create_event(&organizer, &7, &title(&env), &2);
    let created = env.events().all().events()[0]
        .to_xdr(Limits::none())
        .unwrap();
    println!("FIXTURE_CREATED_XDR={}", hex::encode(&created));
    client.reserve(&7, &attendee);
    let reserved = env.events().all().events()[0]
        .to_xdr(Limits::none())
        .unwrap();
    println!("FIXTURE_RESERVED_XDR={}", hex::encode(&reserved));
    client.close_event(&organizer, &7);
    let closed = env.events().all().events()[0]
        .to_xdr(Limits::none())
        .unwrap();
    println!("FIXTURE_CLOSED_XDR={}", hex::encode(&closed));
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../docs/fixtures/events-v1.local.json")).unwrap();
    assert_eq!(
        fixture["events"][0]["contract_event_xdr_hex"],
        hex::encode(&created)
    );
    assert_eq!(
        fixture["events"][1]["contract_event_xdr_hex"],
        hex::encode(&reserved)
    );
    assert_eq!(
        fixture["events"][2]["contract_event_xdr_hex"],
        hex::encode(closed)
    );
}

#[test]
fn archived_event_auto_restores_without_double_booking() {
    let (env, id, client, organizer, attendee) = setup();
    client.create_event(&organizer, &1, &title(&env), &1);
    let original = client.reserve(&1, &attendee);
    let current = env.ledger().sequence();
    env.ledger()
        .set_sequence_number(current + TTL_EXTEND_TO + 1);
    assert_eq!(client.get_event(&1).reserved, 1);
    assert_eq!(client.get_reservation(&1, &attendee), Some(original));
    assert_eq!(
        client.try_reserve(&1, &attendee),
        Err(Ok(Error::AlreadyReserved))
    );
    assert_eq!(
        client.try_reserve(&1, &Address::generate(&env)),
        Err(Ok(Error::CapacityReached))
    );
    assert_eq!(
        client.try_create_event(&organizer, &1, &title(&env), &1),
        Err(Ok(Error::DuplicateEventId))
    );
    let restored_ttl = env.as_contract(&id, || env.storage().persistent().get_ttl(&Key::Event(1)));
    assert!(restored_ttl > 0);
}

#[test]
fn attendee_must_authorize_reservation() {
    let (env, id, client, organizer, attendee) = setup();
    let event_title = title(&env);
    env.mock_auths(&[MockAuth {
        address: &organizer,
        invoke: &MockAuthInvoke {
            contract: &id,
            fn_name: "create_event",
            args: (&organizer, 1_u64, &event_title, 2_u32).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.create_event(&organizer, &1, &event_title, &2);
    env.mock_auths(&[]);
    assert!(client.try_reserve(&1, &attendee).is_err());
    assert_eq!(client.get_event(&1).reserved, 0);
}

#[test]
fn reservation_id_overflow_has_no_partial_write() {
    let (env, id, client, organizer, attendee) = setup();
    client.create_event(&organizer, &1, &title(&env), &1);
    env.as_contract(&id, || {
        env.storage()
            .instance()
            .set(&Key::NextReservationId, &u64::MAX)
    });
    assert_eq!(
        client.try_reserve(&1, &attendee),
        Err(Ok(Error::ArithmeticOverflow))
    );
    assert_eq!(client.get_event(&1).reserved, 0);
    assert!(!client.has_reserved(&1, &attendee));
}

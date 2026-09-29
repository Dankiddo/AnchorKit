#![cfg(test)]

use soroban_sdk::{
    testutils::Address as _,
    symbol_short, Address, Env, String, Symbol,
};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;

use crate::contract::{AnchorKitContract, AnchorKitContractClient};
use crate::sep10_test_util::register_attestor_with_sep10;

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn make_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env
}

fn setup(env: &Env) -> (AnchorKitContractClient, Address, Address, SigningKey) {
    let contract_id = env.register_contract(None, AnchorKitContract);
    let client = AnchorKitContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let attestor = Address::generate(env);
    client.initialize(&admin, &100_u64, &None, &None);
    let sk = SigningKey::generate(&mut OsRng);
    register_attestor_with_sep10(env, &client, &attestor, &admin, &sk);
    (client, admin, attestor, sk)
}

/// Returns the last published event for the given 2-symbol topic prefix.
fn assert_last_event(env: &Env, topic0: Symbol, topic1: Symbol) {
    let events = env.events().all();
    assert!(
        !events.is_empty(),
        "expected at least one event to have been published"
    );
    let (_publisher, topics, _data) = events.get(events.len() - 1).unwrap();
    assert_eq!(topics.len(), 2, "event must have exactly two topic symbols");
    assert_eq!(
        Symbol::try_from_val(env, &topics.get(0).unwrap()).unwrap(),
        topic0,
        "first topic mismatch"
    );
    assert_eq!(
        Symbol::try_from_val(env, &topics.get(1).unwrap()).unwrap(),
        topic1,
        "second topic mismatch"
    );
}

fn assert_event_count(env: &Env, expected: usize) {
    assert_eq!(env.events().all().len(), expected, "unexpected event count");
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[test]
fn test_register_attestor_emits_registered_event() {
    let env = make_env();
    let (client, _, attestor, _) = setup(&env);
    let token = String::from_str(&env, "mock_token");

    let events_before = env.events().all().len();

    client.register_attestor(&attestor, &token, &attestor);

    assert_event_count(&env, events_before + 1);
    assert_last_event(&env, symbol_short!("attestor"), symbol_short!("reg"));
}

#[test]
fn test_revoke_attestor_emits_revoked_event() {
    let env = make_env();
    let (client, _, attestor, _) = setup(&env);

    let events_before = env.events().all().len();

    client.revoke_attestor(&attestor);

    assert_event_count(&env, events_before + 1);
    assert_last_event(&env, symbol_short!("attestor"), symbol_short!("revoked"));
}

#[test]
fn test_set_get_endpoint_happy_path() {
    let env = make_env();
    let (client, _, attestor, _) = setup(&env);
    let endpoint = String::from_str(&env, "https://example.com/api");

    client.set_endpoint(&attestor, &endpoint);

    let retrieved = client.get_endpoint(&attestor);
    assert_eq!(retrieved, endpoint);
}

#[test]
#[should_panic(expected = "AttestorNotRegistered")]
fn test_get_endpoint_not_registered() {
    let env = make_env();
    let (client, _, _, _) = setup(&env);
    let unknown = Address::generate(&env);
    client.get_endpoint(&unknown);
}

#[test]
#[should_panic(expected = "AttestorNotRegistered")]
fn test_set_endpoint_not_attestor() {
    let env = make_env();
    let (client, _, _, _) = setup(&env);
    let unknown = Address::generate(&env);
    let endpoint = String::from_str(&env, "https://example.com");
    client.set_endpoint(&unknown, &endpoint);
}

#[test]
#[should_panic(expected = "InvalidEndpointFormat")]
fn test_set_endpoint_invalid_url() {
    let env = make_env();
    let (client, _, attestor, _) = setup(&env);
    let invalid = String::from_str(&env, "http://invalid.com"); // HTTP
    client.set_endpoint(&attestor, &invalid);
}

#[test]
fn test_endpoint_updated_event() {
    let env = make_env();
    let (client, _, attestor, _) = setup(&env);
    let endpoint = String::from_str(&env, "https://test.com");

    let events_before = env.events().all().len();

    client.set_endpoint(&attestor, &endpoint);

    assert_event_count(&env, events_before + 1);
    assert_last_event(&env, symbol_short!("endpoint"), symbol_short!("updated"));
}
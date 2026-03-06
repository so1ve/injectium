use injectium::{Container, Injectable, cloned, container};

#[derive(Clone, Debug, PartialEq, Eq)]
struct RequestId(String);

#[derive(Injectable)]
struct Service {
    request_id: RequestId,
}

#[derive(Injectable)]
struct MissingService {
    _request_id: RequestId,
}

#[test]
fn derive_injectable_supports_closure_providers() {
    let container = container! {
        providers: [|_c: &Container| RequestId(String::from("factory"))]
    };

    let service = Service::from_container(&container);

    assert_eq!(service.request_id, RequestId(String::from("factory")));
}

#[test]
fn derive_injectable_try_from_container_supports_closure_providers() {
    let container = container! {
        providers: [|_c: &Container| RequestId(String::from("factory"))]
    };

    let service = Service::try_from_container(&container).expect("closure provider should resolve");

    assert_eq!(service.request_id, RequestId(String::from("factory")));
}

#[test]
fn derive_injectable_supports_clone_providers() {
    let container = container! {
        providers: [cloned(RequestId(String::from("singleton")))]
    };

    let service = Service::from_container(&container);

    assert_eq!(service.request_id, RequestId(String::from("singleton")));
}

#[test]
fn derive_injectable_try_from_container_returns_none_when_missing() {
    let container = container! {
        providers: []
    };

    assert!(MissingService::try_from_container(&container).is_none());
}

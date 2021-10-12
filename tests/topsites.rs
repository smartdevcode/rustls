/*
 * These tests check we can handshake with a selection of
 * common hosts.
 *
 * Rules: only hosts that can really handle the traffic.
 * Because we don't go to the same host twice, polite()
 * is not needed.
 */

#[allow(dead_code)]
mod common;

mod online {
use super::common::TlsClient;

fn connect(hostname: &str) -> TlsClient {
  TlsClient::new(hostname)
}

#[test]
fn joe() {
  connect("jbp.io")
    .expect("HTTP/1.1 200 OK")
    .go()
    .unwrap();
}

#[test]
fn google() {
  /* Test we agree chacha20-poly1305 and curve25519 */
  connect("google.com")
    .verbose()
    .expect("HTTP/1.1 ") // currently 302 redirects
    .expect_log("Using ciphersuite TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256")
    .expect_log("ECDHE curve is ECParameters { curve_type: NamedCurve, named_curve: X25519 }")
    .go()
    .unwrap();
}

#[test]
fn github() {
  connect("github.com")
    .expect("HTTP/1.1 200 OK")
    .go()
    .unwrap();
}

#[test]
fn aws() {
  connect("aws.amazon.com")
    .expect("HTTP/1.1 200 OK")
    .go()
    .unwrap();
}

#[test]
fn microsoft() {
  connect("www.microsoft.com")
    .expect("HTTP/1.1 200 OK")
    .go()
    .unwrap();
}

#[test]
fn wikipedia() {
  connect("www.wikipedia.org")
    .expect("HTTP/1.1 200 OK")
    .go()
    .unwrap();
}

#[test]
fn twitter() {
  connect("twitter.com")
    .expect("HTTP/1.1 ")
    .go()
    .unwrap();
}

#[test]
fn facebook() {
  connect("www.facebook.com")
    .expect("HTTP/1.1 ") // also 302s to a 'piss off' page. charming.
    .go()
    .unwrap();
}

#[test]
fn baidu() {
  connect("www.baidu.com")
    .expect("HTTP/1.1 200 OK")
    .go()
    .unwrap();
}

#[test]
fn netflix() {
  connect("www.netflix.com")
    .expect("HTTP/1.1 ")
    .go()
    .unwrap();
}

#[test]
fn stackoverflow() {
  connect("stackoverflow.com")
    .expect("HTTP/1.1 ")
    .go()
    .unwrap();
}

#[test]
fn apple() {
  connect("www.apple.com")
    .expect("HTTP/1.1 200 OK")
    .go()
    .unwrap();
}

#[test]
fn wordpress() {
  connect("wordpress.com")
    .expect("HTTP/1.1 200 OK")
    .go()
    .unwrap();
}

}

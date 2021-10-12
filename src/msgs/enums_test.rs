/// These tests are intended to provide coverage and
/// check panic-safety of relatively unused values.

use super::codec::{Codec, decode_u8, encode_u8, decode_u16, encode_u16};
use super::enums::*;

fn get8<T: Codec>(enum_value: &T) -> u8 {
    let enc = enum_value.get_encoding();
    assert_eq!(enc.len(), 1);
    decode_u8(&enc).unwrap()
}

fn get16<T: Codec>(enum_value: &T) -> u16 {
    let enc = enum_value.get_encoding();
    assert_eq!(enc.len(), 2);
    decode_u16(&enc).unwrap()
}

fn test_enum16<T: Codec>(first: T, last: T) {
    let first_v = get16(&first);
    let last_v = get16(&last);

    for val in first_v..last_v + 1 {
        let mut buf = Vec::new();
        encode_u16(val, &mut buf);
        assert_eq!(buf.len(), 2);

        let t = T::read_bytes(&buf).unwrap();
        assert_eq!(val, get16(&t));
    }
}

fn test_enum8<T: Codec>(first: T, last: T) {
    let first_v = get8(&first);
    let last_v = get8(&last);

    for val in first_v..last_v + 1 {
        let mut buf = Vec::new();
        encode_u8(val, &mut buf);
        assert_eq!(buf.len(), 1);

        let t = T::read_bytes(&buf).unwrap();
        assert_eq!(val, get8(&t));
    }
}

#[test]
fn test_enums() {
    test_enum16::<ProtocolVersion>(ProtocolVersion::SSLv2, ProtocolVersion::TLSv1_2);
    test_enum8::<HashAlgorithm>(HashAlgorithm::NONE, HashAlgorithm::SHA512);
    test_enum8::<SignatureAlgorithm>(SignatureAlgorithm::Anonymous, SignatureAlgorithm::ECDSA);
    test_enum8::<ClientCertificateType>(ClientCertificateType::RSASign,
                                        ClientCertificateType::ECDSAFixedECDH);
    test_enum8::<Compression>(Compression::Null, Compression::LSZ);
    test_enum8::<ContentType>(ContentType::ChangeCipherSpec, ContentType::Heartbeat);
    test_enum8::<HandshakeType>(HandshakeType::HelloRequest,
                                HandshakeType::CertificateStatus);
    test_enum8::<AlertLevel>(AlertLevel::Warning, AlertLevel::Fatal);
    test_enum8::<AlertDescription>(AlertDescription::CloseNotify,
                                   AlertDescription::UnrecognisedName);
    test_enum8::<HeartbeatMessageType>(HeartbeatMessageType::Request,
                                       HeartbeatMessageType::Response);
    test_enum16::<ExtensionType>(ExtensionType::ServerName, ExtensionType::RenegotiationInfo);
    test_enum8::<ServerNameType>(ServerNameType::HostName, ServerNameType::HostName);
    test_enum16::<NamedCurve>(NamedCurve::sect163k1,
                              NamedCurve::arbitrary_explicit_char2_curves);
    test_enum16::<CipherSuite>(CipherSuite::TLS_NULL_WITH_NULL_NULL,
                               CipherSuite::SSL_RSA_FIPS_WITH_3DES_EDE_CBC_SHA);
    test_enum8::<ECPointFormat>(ECPointFormat::Uncompressed,
                                ECPointFormat::ANSIX962CompressedChar2);
    test_enum8::<HeartbeatMode>(HeartbeatMode::PeerAllowedToSend,
                                HeartbeatMode::PeerNotAllowedToSend);
    test_enum8::<ECCurveType>(ECCurveType::ExplicitPrime, ECCurveType::NamedCurve);
}

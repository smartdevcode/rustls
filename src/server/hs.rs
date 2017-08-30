use msgs::enums::{ContentType, HandshakeType, ProtocolVersion};
use msgs::enums::{Compression, NamedGroup, ECPointFormat, CipherSuite};
use msgs::enums::{ExtensionType, AlertDescription};
use msgs::enums::{ClientCertificateType, SignatureScheme, PSKKeyExchangeMode};
use msgs::message::{Message, MessagePayload};
use msgs::base::{Payload, PayloadU8};
use msgs::handshake::{HandshakePayload, SupportedSignatureSchemes};
use msgs::handshake::{HandshakeMessagePayload, ServerHelloPayload, Random};
use msgs::handshake::{ClientHelloPayload, ServerExtension, SessionID};
use msgs::handshake::{ConvertProtocolNameList, ConvertServerNameList};
use msgs::handshake::{NamedGroups, SupportedGroups, ClientExtension};
use msgs::handshake::{ECPointFormatList, SupportedPointFormats};
use msgs::handshake::{ServerECDHParams, DigitallySignedStruct};
use msgs::handshake::{ServerKeyExchangePayload, ECDHEServerKeyExchange};
use msgs::handshake::{CertificateRequestPayload, NewSessionTicketPayload};
use msgs::handshake::{CertificateRequestPayloadTLS13, NewSessionTicketPayloadTLS13};
use msgs::handshake::{HelloRetryRequest, HelloRetryExtension, KeyShareEntry};
use msgs::handshake::{CertificatePayloadTLS13, CertificateEntry};
use msgs::handshake::{CertificateStatus, CertificateExtension};
use msgs::handshake::SupportedMandatedSignatureSchemes;
use msgs::ccs::ChangeCipherSpecPayload;
use msgs::codec::Codec;
use msgs::persist;
use session::SessionSecrets;
use cipher;
use server::ServerSessionImpl;
use key_schedule::{KeySchedule, SecretKind};
use suites;
use hash_hs;
use verify;
use util;
use rand;
use sign;
use error::TLSError;
use handshake::{check_handshake_message, check_message};
use untrusted;
use webpki;

use server::common::{HandshakeDetails, ServerKXDetails, ClientCertDetails};

use ring::constant_time;

macro_rules! extract_handshake(
  ( $m:expr, $t:path ) => (
    match $m.payload {
      MessagePayload::Handshake(ref hsp) => match hsp.payload {
        $t(ref hm) => Some(hm),
        _ => None
      },
      _ => None
    }
  )
);

type CheckResult = Result<(), TLSError>;
type StateResult = Result<Box<State + Send>, TLSError>;

pub trait State {
    fn check_message(&self, m: &Message) -> CheckResult;
    fn handle(self: Box<Self>, sess: &mut ServerSessionImpl, m: Message) -> StateResult;
}

fn incompatible(sess: &mut ServerSessionImpl, why: &str) -> TLSError {
    sess.common.send_fatal_alert(AlertDescription::HandshakeFailure);
    TLSError::PeerIncompatibleError(why.to_string())
}

fn illegal_param(sess: &mut ServerSessionImpl, why: &str) -> TLSError {
    sess.common.send_fatal_alert(AlertDescription::IllegalParameter);
    TLSError::PeerMisbehavedError(why.to_string())
}

fn decode_error(sess: &mut ServerSessionImpl, why: &str) -> TLSError {
    sess.common.send_fatal_alert(AlertDescription::DecodeError);
    TLSError::PeerMisbehavedError(why.to_string())
}

fn can_resume(sess: &ServerSessionImpl,
              handshake: &HandshakeDetails,
              resumedata: &Option<persist::ServerSessionValue>) -> bool {
    // The RFCs underspecify what happens if we try to resume to
    // an unoffered/varying suite.  We merely don't resume in weird cases.
    //
    // RFC 6066 says "A server that implements this extension MUST NOT accept
    // the request to resume the session if the server_name extension contains
    // a different name. Instead, it proceeds with a full handshake to
    // establish a new session."

    if let Some(ref resume) = *resumedata {
        resume.cipher_suite == sess.common.get_suite().suite &&
            (resume.extended_ms == handshake.using_ems ||
             (resume.extended_ms && !handshake.using_ems)) &&
            same_dns_name_or_both_none(resume.sni.as_ref(), sess.sni.as_ref())
    } else {
        false
    }
}

// Require an exact match for the purpose of comparing SNI DNS Names from two
// client hellos, even though a case-insensitive comparison might also be OK.
fn same_dns_name_or_both_none(a: Option<&webpki::DNSName>,
                              b: Option<&webpki::DNSName>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => {
            let a: &str = a.as_ref().into();
            let b: &str = b.as_ref().into();
            a == b
        },
        (None, None) => true,
        _ => false,
    }
}

// Changing the keys must not span any fragmented handshake
// messages.  Otherwise the defragmented messages will have
// been protected with two different record layer protections,
// which is illegal.  Not mentioned in RFC.
fn check_aligned_handshake(sess: &mut ServerSessionImpl) -> Result<(), TLSError> {
    if !sess.common.handshake_joiner.is_empty() {
        Err(illegal_param(sess, "keys changed with pending hs fragment"))
    } else {
        Ok(())
    }
}

pub struct ExpectClientHello {
    handshake: HandshakeDetails,
    done_retry: bool,
    send_cert_status: bool,
    send_sct: bool,
    send_ticket: bool,
}

impl ExpectClientHello {
    pub fn new(perhaps_client_auth: bool) -> ExpectClientHello {
        let mut ret = ExpectClientHello {
            handshake: HandshakeDetails::new(),
            done_retry: false,
            send_cert_status: false,
            send_sct: false,
            send_ticket: false,
        };

        if perhaps_client_auth {
            ret.handshake.transcript.set_client_auth_enabled();
        }

        ret
    }

    fn into_expect_tls12_ccs(self) -> Box<State + Send> {
        Box::new(ExpectTLS12CCS {
            handshake: self.handshake,
            resuming: true,
            send_ticket: self.send_ticket,
        })
    }

    fn into_expect_retried_client_hello(self) -> Box<State + Send> {
        Box::new(ExpectClientHello {
            handshake: self.handshake,
            done_retry: true,
            send_cert_status: self.send_cert_status,
            send_sct: self.send_sct,
            send_ticket: self.send_ticket,
        })
    }

    fn into_expect_tls13_certificate(self) -> Box<State + Send> {
        Box::new(ExpectTLS13Certificate {
            handshake: self.handshake,
            send_ticket: self.send_ticket,
        })
    }

    fn into_expect_tls13_finished(self) -> Box<State + Send> {
        Box::new(ExpectTLS13Finished {
            handshake: self.handshake,
            send_ticket: self.send_ticket,
        })
    }

    fn into_expect_tls12_certificate(self, kx: suites::KeyExchange) -> Box<State + Send> {
        Box::new(ExpectTLS12Certificate {
            handshake: self.handshake,
            server_kx: ServerKXDetails::new(kx),
            send_ticket: self.send_ticket,
        })
    }

    fn into_expect_tls12_client_kx(self, kx: suites::KeyExchange) -> Box<State + Send> {
        Box::new(ExpectTLS12ClientKX {
            handshake: self.handshake,
            server_kx: ServerKXDetails::new(kx),
            client_cert: None,
            send_ticket: self.send_ticket,
        })
    }

    fn process_extensions(&mut self,
                          sess: &mut ServerSessionImpl,
                          server_key: Option<&mut sign::CertifiedKey>,
                          hello: &ClientHelloPayload,
                          for_resume: bool)
                          -> Result<Vec<ServerExtension>, TLSError> {
        let mut ret = Vec::new();

        // ALPN
        let our_protocols = &sess.config.alpn_protocols;
        let maybe_their_protocols = hello.get_alpn_extension();
        if let Some(their_protocols) = maybe_their_protocols {
            let their_proto_strings = their_protocols.to_strings();

            if their_proto_strings.contains(&"".to_string()) {
                return Err(TLSError::PeerMisbehavedError("client offered empty ALPN protocol"
                    .to_string()));
            }

            sess.alpn_protocol = util::first_in_both(our_protocols, &their_proto_strings);
            if let Some(ref selected_protocol) = sess.alpn_protocol {
                info!("Chosen ALPN protocol {:?}", selected_protocol);
                ret.push(ServerExtension::make_alpn(selected_protocol.clone()));
            }
        }

        // SNI
        if !for_resume && hello.get_sni_extension().is_some() {
            ret.push(ServerExtension::ServerNameAck);
        }

        // Send status_request response if we have one.  This is not allowed
        // if we're resuming, and is only triggered if we have an OCSP response
        // to send.
        if !for_resume &&
           hello.find_extension(ExtensionType::StatusRequest).is_some() &&
           server_key.is_some() &&
           server_key.as_ref().unwrap().has_ocsp() {
            self.send_cert_status = true;

            if !sess.common.is_tls13() {
                // Only TLS1.2 sends confirmation in ServerHello
                ret.push(ServerExtension::CertificateStatusAck);
            }
        }

        if !for_resume &&
           hello.find_extension(ExtensionType::SCT).is_some() &&
           server_key.is_some() &&
           server_key.as_ref().unwrap().has_sct_list() {
            self.send_sct = true;

            if !sess.common.is_tls13() {
                let sct_list = server_key
                    .unwrap()
                    .take_sct_list()
                    .unwrap();
                ret.push(ServerExtension::make_sct(sct_list));
            }
        }

        if !sess.common.is_tls13() {
            // Renegotiation.
            // (We don't do reneg at all, but would support the secure version if we did.)
            let secure_reneg_offered =
                hello.find_extension(ExtensionType::RenegotiationInfo).is_some() ||
                hello.cipher_suites.contains(&CipherSuite::TLS_EMPTY_RENEGOTIATION_INFO_SCSV);

            if secure_reneg_offered {
                ret.push(ServerExtension::make_empty_renegotiation_info());
            }

            // Tickets:
            // If we get any SessionTicket extension and have tickets enabled,
            // we send an ack.
            if hello.find_extension(ExtensionType::SessionTicket).is_some() &&
               sess.config.ticketer.enabled() {
                self.send_ticket = true;
                ret.push(ServerExtension::SessionTicketAck);
            }

            // Confirm use of EMS if offered.
            if self.handshake.using_ems {
                ret.push(ServerExtension::ExtendedMasterSecretAck);
            }

        }

        Ok(ret)
    }

    fn check_binder(&self,
                    sess: &mut ServerSessionImpl,
                    client_hello: &Message,
                    psk: &[u8],
                    binder: &[u8])
                    -> bool {
        let binder_plaintext = match client_hello.payload {
            MessagePayload::Handshake(ref hmp) => hmp.get_encoding_for_binder_signing(),
            _ => unreachable!(),
        };

        let suite_hash = sess.common.get_suite().get_hash();
        let handshake_hash = self.handshake.transcript.get_hash_given(suite_hash, &binder_plaintext);

        let mut empty_hash_ctx = hash_hs::HandshakeHash::new();
        empty_hash_ctx.start_hash(suite_hash);
        let empty_hash = empty_hash_ctx.get_current_hash();

        let mut key_schedule = KeySchedule::new(suite_hash);
        key_schedule.input_secret(psk);
        let base_key = key_schedule.derive(SecretKind::ResumptionPSKBinderKey, &empty_hash);
        let real_binder = key_schedule.sign_verify_data(&base_key, &handshake_hash);

        constant_time::verify_slices_are_equal(&real_binder, binder).is_ok()
    }

    fn emit_server_hello_tls13(&mut self,
                               sess: &mut ServerSessionImpl,
                               share: &KeyShareEntry,
                               chosen_psk_idx: Option<usize>,
                               resuming_psk: Option<Vec<u8>>)
                               -> Result<(), TLSError> {
        let mut extensions = Vec::new();

        // Do key exchange
        let kxr = suites::KeyExchange::start_ecdhe(share.group)
            .and_then(|kx| kx.complete(&share.payload.0))
            .ok_or_else(|| TLSError::PeerMisbehavedError("key exchange failed".to_string()))?;

        let kse = KeyShareEntry::new(share.group, &kxr.pubkey);
        extensions.push(ServerExtension::KeyShare(kse));

        if let Some(psk_idx) = chosen_psk_idx {
            extensions.push(ServerExtension::PresharedKey(psk_idx as u16));
        }

        let sh = Message {
            typ: ContentType::Handshake,
            version: ProtocolVersion::TLSv1_0,
            payload: MessagePayload::Handshake(HandshakeMessagePayload {
                typ: HandshakeType::ServerHello,
                payload: HandshakePayload::ServerHello(ServerHelloPayload {
                    server_version: ProtocolVersion::Unknown(0x7f12),
                    random: Random::from_slice(&self.handshake.randoms.server),
                    session_id: SessionID::empty(),
                    cipher_suite: sess.common.get_suite().suite,
                    compression_method: Compression::Null,
                    extensions: extensions,
                }),
            }),
        };

        check_aligned_handshake(sess)?;

        debug!("sending server hello {:?}", sh);
        self.handshake.transcript.add_message(&sh);
        sess.common.send_msg(sh, false);

        // Start key schedule
        let suite = sess.common.get_suite();
        let mut key_schedule = KeySchedule::new(suite.get_hash());
        if let Some(psk) = resuming_psk {
            key_schedule.input_secret(&psk);
        } else {
            key_schedule.input_empty();
        }
        key_schedule.input_secret(&kxr.premaster_secret);

        let handshake_hash = self.handshake.transcript.get_current_hash();
        let write_key = key_schedule.derive(SecretKind::ServerHandshakeTrafficSecret, &handshake_hash);
        let read_key = key_schedule.derive(SecretKind::ClientHandshakeTrafficSecret, &handshake_hash);
        sess.common.set_message_encrypter(cipher::new_tls13_write(suite, &write_key));
        sess.common.set_message_decrypter(cipher::new_tls13_read(suite, &read_key));
        key_schedule.current_client_traffic_secret = read_key;
        key_schedule.current_server_traffic_secret = write_key;
        sess.common.set_key_schedule(key_schedule);

        Ok(())
    }

    fn emit_hello_retry_request(&mut self,
                                sess: &mut ServerSessionImpl,
                                group: NamedGroup) {
        let mut req = HelloRetryRequest {
            server_version: ProtocolVersion::Unknown(0x7f12),
            extensions: Vec::new(),
        };

        req.extensions.push(HelloRetryExtension::KeyShare(group));

        let m = Message {
            typ: ContentType::Handshake,
            version: ProtocolVersion::TLSv1_0,
            payload: MessagePayload::Handshake(HandshakeMessagePayload {
                typ: HandshakeType::HelloRetryRequest,
                payload: HandshakePayload::HelloRetryRequest(req),
            }),
        };

        debug!("Requesting retry {:?}", m);
        self.handshake.transcript.add_message(&m);
        sess.common.send_msg(m, false);
    }

    fn emit_encrypted_extensions(&mut self,
                                 sess: &mut ServerSessionImpl,
                                 server_key: &mut sign::CertifiedKey,
                                 hello: &ClientHelloPayload,
                                 for_resume: bool)
                                 -> Result<(), TLSError> {
        let encrypted_exts = self.process_extensions(sess, Some(server_key), hello, for_resume)?;
        let ee = Message {
            typ: ContentType::Handshake,
            version: ProtocolVersion::TLSv1_3,
            payload: MessagePayload::Handshake(HandshakeMessagePayload {
                typ: HandshakeType::EncryptedExtensions,
                payload: HandshakePayload::EncryptedExtensions(encrypted_exts),
            }),
        };

        debug!("sending encrypted extensions {:?}", ee);
        self.handshake.transcript.add_message(&ee);
        sess.common.send_msg(ee, true);
        Ok(())
    }

    fn emit_certificate_req_tls13(&mut self, sess: &mut ServerSessionImpl) -> bool {
        if !sess.config.client_auth_offer {
            return false;
        }

        let names = sess.config.client_auth_roots.get_subjects();

        let cr = CertificateRequestPayloadTLS13 {
            context: PayloadU8::empty(),
            sigschemes: SupportedSignatureSchemes::supported_verify(),
            canames: names,
            extensions: Vec::new(),
        };

        let m = Message {
            typ: ContentType::Handshake,
            version: ProtocolVersion::TLSv1_3,
            payload: MessagePayload::Handshake(HandshakeMessagePayload {
                typ: HandshakeType::CertificateRequest,
                payload: HandshakePayload::CertificateRequestTLS13(cr),
            }),
        };

        debug!("Sending CertificateRequest {:?}", m);
        self.handshake.transcript.add_message(&m);
        sess.common.send_msg(m, true);
        true
    }

    fn emit_certificate_tls13(&mut self,
                              sess: &mut ServerSessionImpl,
                              server_key: &mut sign::CertifiedKey) {
        let mut cert_body = CertificatePayloadTLS13::new();

        let (certs, ocsp, sct_list) = {
            let ck = server_key;
            (ck.take_cert(), ck.take_ocsp(), ck.take_sct_list())
        };

        for cert in certs {
            let entry = CertificateEntry {
                cert: cert,
                exts: Vec::new(),
            };

            cert_body.list.push(entry);
        }

        // Apply OCSP response to first certificate (we don't support OCSP
        // except for leaf certs).
        if self.send_cert_status &&
           ocsp.is_some() &&
           !cert_body.list.is_empty() {
            let first_entry = cert_body.list.first_mut().unwrap();
            let cst = CertificateStatus::new(ocsp.unwrap());
            first_entry.exts.push(CertificateExtension::CertificateStatus(cst));
        }

        // Likewise, SCT
        if self.send_sct &&
           sct_list.is_some() &&
           !cert_body.list.is_empty() {
            let first_entry = cert_body.list.first_mut().unwrap();
            first_entry.exts.push(CertificateExtension::make_sct(sct_list.unwrap()));
        }

        let c = Message {
            typ: ContentType::Handshake,
            version: ProtocolVersion::TLSv1_3,
            payload: MessagePayload::Handshake(HandshakeMessagePayload {
                typ: HandshakeType::Certificate,
                payload: HandshakePayload::CertificateTLS13(cert_body),
            }),
        };

        debug!("sending certificate {:?}", c);
        self.handshake.transcript.add_message(&c);
        sess.common.send_msg(c, true);
    }

    fn emit_certificate_verify_tls13(&mut self,
                                     sess: &mut ServerSessionImpl,
                                     server_key: &mut sign::CertifiedKey,
                                     schemes: &[SignatureScheme])
                                     -> Result<(), TLSError> {
        let mut message = Vec::new();
        message.resize(64, 0x20u8);
        message.extend_from_slice(b"TLS 1.3, server CertificateVerify\x00");
        message.extend_from_slice(&self.handshake.transcript.get_current_hash());

        let signing_key = &server_key.key;
        let signer = signing_key.choose_scheme(schemes)
            .ok_or_else(|| TLSError::PeerIncompatibleError("no overlapping sigschemes".to_string()))?;

        let scheme = signer.get_scheme();
        let sig = signer.sign(&message)?;

        let cv = DigitallySignedStruct::new(scheme, sig);

        let m = Message {
            typ: ContentType::Handshake,
            version: ProtocolVersion::TLSv1_3,
            payload: MessagePayload::Handshake(HandshakeMessagePayload {
                typ: HandshakeType::CertificateVerify,
                payload: HandshakePayload::CertificateVerify(cv),
            }),
        };

        debug!("sending certificate-verify {:?}", m);
        self.handshake.transcript.add_message(&m);
        sess.common.send_msg(m, true);
        Ok(())
    }

    fn emit_finished_tls13(&mut self, sess: &mut ServerSessionImpl) {
        let handshake_hash = self.handshake.transcript.get_current_hash();
        let verify_data = sess.common
            .get_key_schedule()
            .sign_finish(SecretKind::ServerHandshakeTrafficSecret, &handshake_hash);
        let verify_data_payload = Payload::new(verify_data);

        let m = Message {
            typ: ContentType::Handshake,
            version: ProtocolVersion::TLSv1_3,
            payload: MessagePayload::Handshake(HandshakeMessagePayload {
                typ: HandshakeType::Finished,
                payload: HandshakePayload::Finished(verify_data_payload),
            }),
        };

        debug!("sending finished {:?}", m);
        self.handshake.transcript.add_message(&m);
        self.handshake.hash_at_server_fin = self.handshake.transcript.get_current_hash();
        sess.common.send_msg(m, true);

        // Now move to application data keys.
        sess.common.get_mut_key_schedule().input_empty();
        let write_key = sess.common
            .get_key_schedule()
            .derive(SecretKind::ServerApplicationTrafficSecret,
                    &self.handshake.hash_at_server_fin);
        let suite = sess.common.get_suite();
        sess.common.set_message_encrypter(cipher::new_tls13_write(suite, &write_key));
        sess.common
            .get_mut_key_schedule()
            .current_server_traffic_secret = write_key;
    }

    fn emit_server_hello(&mut self,
                         sess: &mut ServerSessionImpl,
                         server_key: Option<&mut sign::CertifiedKey>,
                         hello: &ClientHelloPayload,
                         for_resume: bool)
                         -> Result<(), TLSError> {
        let extensions = self.process_extensions(sess, server_key, hello, for_resume)?;

        if self.handshake.session_id.is_empty() {
            let sessid = sess.config
                .session_storage
                .generate();
            self.handshake.session_id = sessid;
        }

        let sh = Message {
            typ: ContentType::Handshake,
            version: ProtocolVersion::TLSv1_2,
            payload: MessagePayload::Handshake(HandshakeMessagePayload {
                typ: HandshakeType::ServerHello,
                payload: HandshakePayload::ServerHello(ServerHelloPayload {
                    server_version: ProtocolVersion::TLSv1_2,
                    random: Random::from_slice(&self.handshake.randoms.server),
                    session_id: self.handshake.session_id,
                    cipher_suite: sess.common.get_suite().suite,
                    compression_method: Compression::Null,
                    extensions: extensions,
                }),
            }),
        };

        debug!("sending server hello {:?}", sh);
        self.handshake.transcript.add_message(&sh);
        sess.common.send_msg(sh, false);
        Ok(())
    }

    fn emit_certificate(&mut self,
                        sess: &mut ServerSessionImpl,
                        server_certkey: &mut sign::CertifiedKey) {
        let cert_chain = server_certkey.take_cert();

        let c = Message {
            typ: ContentType::Handshake,
            version: ProtocolVersion::TLSv1_2,
            payload: MessagePayload::Handshake(HandshakeMessagePayload {
                typ: HandshakeType::Certificate,
                payload: HandshakePayload::Certificate(cert_chain),
            }),
        };

        self.handshake.transcript.add_message(&c);
        sess.common.send_msg(c, false);
    }

    fn emit_cert_status(&mut self,
                        sess: &mut ServerSessionImpl,
                        server_certkey: &mut sign::CertifiedKey) {
        if !self.send_cert_status ||
           !server_certkey.has_ocsp() {
            return;
        }

        let ocsp = server_certkey.take_ocsp();
        let st = CertificateStatus::new(ocsp.unwrap());

        let c = Message {
            typ: ContentType::Handshake,
            version: ProtocolVersion::TLSv1_2,
            payload: MessagePayload::Handshake(HandshakeMessagePayload {
                typ: HandshakeType::CertificateStatus,
                payload: HandshakePayload::CertificateStatus(st)
            }),
        };

        self.handshake.transcript.add_message(&c);
        sess.common.send_msg(c, false);
    }

    fn emit_server_kx(&mut self,
                      sess: &mut ServerSessionImpl,
                      sigscheme: SignatureScheme,
                      group: &NamedGroup,
                      server_certkey: &mut sign::CertifiedKey)
                      -> Result<suites::KeyExchange, TLSError> {
        let kx = sess.common.get_suite()
            .start_server_kx(*group)
            .ok_or_else(|| TLSError::PeerMisbehavedError("key exchange failed".to_string()))?;
        let secdh = ServerECDHParams::new(group, &kx.pubkey);

        let mut msg = Vec::new();
        msg.extend(&self.handshake.randoms.client);
        msg.extend(&self.handshake.randoms.server);
        secdh.encode(&mut msg);

        let signing_key = &server_certkey.key;
        let sig = signing_key.choose_scheme(&[sigscheme])
            .ok_or_else(|| TLSError::General("incompatible signing key".to_string()))
            .and_then(|signer| signer.sign(&msg))?;

        let skx = ServerKeyExchangePayload::ECDHE(ECDHEServerKeyExchange {
            params: secdh,
            dss: DigitallySignedStruct::new(sigscheme, sig),
        });

        let m = Message {
            typ: ContentType::Handshake,
            version: ProtocolVersion::TLSv1_2,
            payload: MessagePayload::Handshake(HandshakeMessagePayload {
                typ: HandshakeType::ServerKeyExchange,
                payload: HandshakePayload::ServerKeyExchange(skx),
            }),
        };

        self.handshake.transcript.add_message(&m);
        sess.common.send_msg(m, false);
        Ok(kx)
    }

    fn emit_certificate_req(&mut self, sess: &mut ServerSessionImpl) -> bool {
        if !sess.config.client_auth_offer {
            return false;
        }

        let names = sess.config.client_auth_roots.get_subjects();

        let cr = CertificateRequestPayload {
            certtypes: vec![ ClientCertificateType::RSASign,
                         ClientCertificateType::ECDSASign ],
            sigschemes: SupportedSignatureSchemes::supported_verify(),
            canames: names,
        };

        let m = Message {
            typ: ContentType::Handshake,
            version: ProtocolVersion::TLSv1_2,
            payload: MessagePayload::Handshake(HandshakeMessagePayload {
                typ: HandshakeType::CertificateRequest,
                payload: HandshakePayload::CertificateRequest(cr),
            }),
        };

        debug!("Sending CertificateRequest {:?}", m);
        self.handshake.transcript.add_message(&m);
        sess.common.send_msg(m, false);
        true
    }

    fn emit_server_hello_done(&mut self, sess: &mut ServerSessionImpl) {
        let m = Message {
            typ: ContentType::Handshake,
            version: ProtocolVersion::TLSv1_2,
            payload: MessagePayload::Handshake(HandshakeMessagePayload {
                typ: HandshakeType::ServerHelloDone,
                payload: HandshakePayload::ServerHelloDone,
            }),
        };

        self.handshake.transcript.add_message(&m);
        sess.common.send_msg(m, false);
    }

    fn start_resumption(mut self,
                        sess: &mut ServerSessionImpl,
                        client_hello: &ClientHelloPayload,
                        sni: Option<&webpki::DNSName>,
                        id: &SessionID,
                        resumedata: persist::ServerSessionValue)
                        -> StateResult {
        info!("Resuming session");

        if resumedata.extended_ms && !self.handshake.using_ems {
            return Err(illegal_param(sess, "refusing to resume without ems"));
        }

        self.handshake.session_id = *id;
        self.emit_server_hello(sess, None, client_hello, true)?;

        let hashalg = sess.common.get_suite().get_hash();
        sess.secrets = Some(SessionSecrets::new_resume(&self.handshake.randoms,
                                                       hashalg,
                                                       &resumedata.master_secret.0));
        sess.start_encryption_tls12();
        sess.client_cert_chain = resumedata.client_cert_chain;

        if self.send_ticket {
            emit_ticket(&mut self.handshake, sess);
        }
        emit_ccs(sess);
        emit_finished(&mut self.handshake, sess);

        assert!(same_dns_name_or_both_none(sni, sess.get_sni()));

        Ok(self.into_expect_tls12_ccs())
    }

    fn handle_client_hello_tls13(mut self,
                                 sess: &mut ServerSessionImpl,
                                 mut server_key: sign::CertifiedKey,
                                 chm: &Message)
                                 -> StateResult {
        let client_hello = extract_handshake!(chm, HandshakePayload::ClientHello).unwrap();

        if client_hello.compression_methods.len() != 1 {
            return Err(illegal_param(sess, "client offered wrong compressions"));
        }

        let groups_ext = client_hello.get_namedgroups_extension()
            .ok_or_else(|| incompatible(sess, "client didn't describe groups"))?;

        let mut sigschemes_ext = client_hello.get_sigalgs_extension()
            .ok_or_else(|| incompatible(sess, "client didn't describe sigschemes"))?
            .clone();

        let tls13_schemes = SupportedSignatureSchemes::supported_sign_tls13();
        sigschemes_ext.retain(|scheme| tls13_schemes.contains(scheme));

        let shares_ext = client_hello.get_keyshare_extension()
            .ok_or_else(|| incompatible(sess, "client didn't send keyshares"))?;

        if client_hello.has_keyshare_extension_with_duplicates() {
            return Err(illegal_param(sess, "client sent duplicate keyshares"));
        }

        let share_groups: Vec<NamedGroup> = shares_ext.iter()
            .map(|share| share.group)
            .collect();

        let chosen_group = util::first_in_both(&NamedGroups::supported(), &share_groups);
        if chosen_group.is_none() {
            // We don't have a suitable key share.  Choose a suitable group and
            // send a HelloRetryRequest.
            let retry_group_maybe = util::first_in_both(&NamedGroups::supported(), groups_ext);
            self.handshake.transcript.add_message(chm);

            if let Some(group) = retry_group_maybe {
                if self.done_retry {
                    return Err(illegal_param(sess, "did not follow retry request"));
                } else {
                    self.emit_hello_retry_request(sess, group);
                    return Ok(self.into_expect_retried_client_hello());
                }
            } else {
                return Err(incompatible(sess, "no kx group overlap with client"));
            }
        }

        let sni = self.get_sni_dns_name(sess, client_hello)?;
        self.cross_check_certificate_and_save_sni(sess, sni, &server_key)?;

        let chosen_group = chosen_group.unwrap();
        let chosen_share = shares_ext.iter()
            .find(|share| share.group == chosen_group)
            .unwrap();

        let mut chosen_psk_index = None;
        let mut resuming_psk = None;
        if let Some(psk_offer) = client_hello.get_psk() {
            if !client_hello.check_psk_ext_is_last() {
                return Err(illegal_param(sess, "psk extension in wrong position"));
            }

            if psk_offer.binders.is_empty() {
                return Err(decode_error(sess, "psk extension missing binder"));
            }

            if psk_offer.binders.len() != psk_offer.identities.len() {
                return Err(illegal_param(sess, "psk extension mismatched ids/binders"));
            }

            for (i, psk_id) in psk_offer.identities.iter().enumerate() {
                let maybe_resume = sess.config
                    .ticketer
                    .decrypt(&psk_id.identity.0)
                    .and_then(|plain| persist::ServerSessionValue::read_bytes(&plain));

                if !can_resume(sess, &self.handshake, &maybe_resume) {
                    continue;
                }

                let resume = maybe_resume.unwrap();

                if !self.check_binder(sess, chm, &resume.master_secret.0, &psk_offer.binders[i].0) {
                    sess.common.send_fatal_alert(AlertDescription::DecryptError);
                    return Err(TLSError::PeerMisbehavedError("client sent wrong binder".to_string()));
                }

                chosen_psk_index = Some(i);
                resuming_psk = Some(resume.master_secret.0);
                break;
            }
        }

        if !client_hello.psk_mode_offered(PSKKeyExchangeMode::PSK_DHE_KE) {
            warn!("Resumption ignored, DHE_KE not offered");
            self.send_ticket = false;
            chosen_psk_index = None;
            resuming_psk = None;
        } else {
            self.send_ticket = true;
        }

        let full_handshake = resuming_psk.is_none();
        self.handshake.transcript.add_message(chm);
        self.emit_server_hello_tls13(sess, chosen_share, chosen_psk_index, resuming_psk)?;
        self.emit_encrypted_extensions(sess, &mut server_key, client_hello, !full_handshake)?;

        let doing_client_auth = if full_handshake {
            let client_auth = self.emit_certificate_req_tls13(sess);
            self.emit_certificate_tls13(sess, &mut server_key);
            self.emit_certificate_verify_tls13(sess, &mut server_key, &sigschemes_ext)?;
            client_auth
        } else {
            false
        };

        check_aligned_handshake(sess)?;
        self.emit_finished_tls13(sess);

        if doing_client_auth {
            Ok(self.into_expect_tls13_certificate())
        } else {
            Ok(self.into_expect_tls13_finished())
        }
    }

    fn get_sni_dns_name(&self, sess: &mut ServerSessionImpl,
                        client_hello: &ClientHelloPayload)
                        -> Result<Option<webpki::DNSName>, TLSError> {
        match client_hello.get_sni_extension()
                          .and_then(|sni| sni.get_hostname())
                          .and_then(|sni| sni.get_hostname_str()) {
            Some(sni) =>
                webpki::DNSNameRef::try_from_ascii_str(sni)
                    .map(|dns_name_ref| Some(webpki::DNSName::from(dns_name_ref)))
                    .map_err(|()| illegal_param(sess, "ClientHello SNI DNS name is invalid.")),
            None => Ok(None),
        }
    }

    fn cross_check_certificate_and_save_sni(&self,
                                            sess: &mut ServerSessionImpl,
                                            sni: Option<webpki::DNSName>,
                                            certkey: &sign::CertifiedKey) -> Result<(), TLSError> {
        // Always reject an empty certificate chain.
        let end_entity_cert = certkey.end_entity_cert().map_err(|()| {
            sess.common.send_fatal_alert(AlertDescription::InternalError);
            TLSError::General("no end-entity certificate in certificate chain".to_string())
        })?;

        // Reject syntactically-invalid end-entity certificates.
        let end_entity_cert = webpki::EndEntityCert::from(
              untrusted::Input::from(end_entity_cert.as_ref())).map_err(|_| {
            sess.common.send_fatal_alert(AlertDescription::InternalError);
            TLSError::General(
                "end-entity certificate in certificate chain is syntactically invalid".to_string())
        })?;

        if let Some(ref sni) = sni {
            // If SNI was offered then the certificate must be valid for
            // that hostname. Note that this doesn't fully validate that the
            // certificate is valid; it only validates that the name is one
            // that the certificate is valid for, if the certificate is
            // valid. Indirectly, this also verifies that the SNI is a
            // syntactically-valid hostname, according to Web PKI rules,
            // which may differ from DNS and/or URL rules.
            if !end_entity_cert.verify_is_valid_for_dns_name(sni.as_ref()).is_ok() {
                sess.common.send_fatal_alert(AlertDescription::InternalError);
                return Err(TLSError::General(
                    "The server certificate is not valid for the given SNI name".to_string()));
            }

            // Save the SNI into the session after it's been validated.
            sess.set_sni(sni.clone());
        }

        assert!(same_dns_name_or_both_none(sni.as_ref(), sess.get_sni()));
        Ok(())
    }
}

impl State for ExpectClientHello {
    fn check_message(&self, m: &Message) -> CheckResult {
        check_handshake_message(m, &[HandshakeType::ClientHello])
    }

    fn handle(mut self: Box<Self>, sess: &mut ServerSessionImpl, m: Message) -> StateResult {
        let client_hello = extract_handshake!(m, HandshakePayload::ClientHello).unwrap();
        let tls13_enabled = sess.config.versions.contains(&ProtocolVersion::TLSv1_3);
        let tls12_enabled = sess.config.versions.contains(&ProtocolVersion::TLSv1_2);
        debug!("we got a clienthello {:?}", client_hello);

        if client_hello.client_version.get_u16() < ProtocolVersion::TLSv1_2.get_u16() {
            sess.common.send_fatal_alert(AlertDescription::ProtocolVersion);
            return Err(TLSError::PeerIncompatibleError("client does not support TLSv1_2".to_string()));
        }

        if !client_hello.compression_methods.contains(&Compression::Null) {
            sess.common.send_fatal_alert(AlertDescription::IllegalParameter);
            return Err(TLSError::PeerIncompatibleError("client did not offer Null compression"
                .to_string()));
        }

        if client_hello.has_duplicate_extension() {
            return Err(decode_error(sess, "client sent duplicate extensions"));
        }

        // Are we doing TLS1.3?
        let maybe_versions_ext = client_hello.get_versions_extension();
        if let Some(versions) = maybe_versions_ext {
            if versions.contains(&ProtocolVersion::Unknown(0x7f12)) && tls13_enabled {
                sess.common.negotiated_version = Some(ProtocolVersion::TLSv1_3);
            } else if !versions.contains(&ProtocolVersion::TLSv1_2) || !tls12_enabled {
                sess.common.send_fatal_alert(AlertDescription::ProtocolVersion);
                return Err(incompatible(sess, "TLS1.2 not offered/enabled"));
            }
        } else if !tls12_enabled && tls13_enabled {
            sess.common.send_fatal_alert(AlertDescription::ProtocolVersion);
            return Err(incompatible(sess, "Server requires TLS1.3, but client omitted versions ext"));
        }

        if sess.common.negotiated_version == None {
            sess.common.negotiated_version = Some(ProtocolVersion::TLSv1_2);
        }

        // Common to TLS1.2 and TLS1.3: ciphersuite and certificate selection.
        let default_sigschemes_ext = SupportedSignatureSchemes::default();

        // Extract and validate the SNI DNS name, if any, before giving it to
        // the cert resolver. In particular, if it is invalid then we should
        // send an Illegal Parameter alert instead of the Internal Error alert
        // (or whatever) that we'd send if this were checked later or in a
        // different way.
        let sni = self.get_sni_dns_name(sess, client_hello)?;

        let sigschemes_ext = client_hello.get_sigalgs_extension()
          .unwrap_or(&default_sigschemes_ext);

        // Choose a certificate.
        let mut certkey = {
            let sni_str: Option<&str> =
                sni.as_ref().map(|dns_name| dns_name.as_ref().into());
            debug!("sni {:?}", sni_str);
            debug!("sig schemes {:?}", sigschemes_ext);
            let certkey = sess.config.cert_resolver.resolve(sni_str, sigschemes_ext);
            certkey.ok_or_else(|| {
                sess.common.send_fatal_alert(AlertDescription::AccessDenied);
                TLSError::General("no server certificate chain resolved".to_string())
            })?
        };

        // Reduce our supported ciphersuites by the certificate.
        // (no-op for TLS1.3)
        let suitable_suites = suites::reduce_given_sigalg(&sess.config.ciphersuites,
                                                          &certkey.key.algorithm());

        // And version
        let protocol_version = sess.common.negotiated_version.unwrap();
        let suitable_suites = suites::reduce_given_version(&suitable_suites, protocol_version);

        let maybe_ciphersuite = if sess.config.ignore_client_order {
            suites::choose_ciphersuite_preferring_server(&client_hello.cipher_suites, &suitable_suites)
        } else {
            suites::choose_ciphersuite_preferring_client(&client_hello.cipher_suites, &suitable_suites)
        };

        if maybe_ciphersuite.is_none() {
            return Err(incompatible(sess, "no ciphersuites in common"));
        }

        info!("decided upon suite {:?}", maybe_ciphersuite.as_ref().unwrap());
        sess.common.set_suite(maybe_ciphersuite.unwrap());

        // Start handshake hash.
        if !self.handshake.transcript.start_hash(sess.common.get_suite().get_hash()) {
            sess.common.send_fatal_alert(AlertDescription::IllegalParameter);
            return Err(TLSError::PeerIncompatibleError("hash differed on retry"
                .to_string()));
        }

        if sess.common.is_tls13() {
            return self.handle_client_hello_tls13(sess, certkey, &m);
        }

        // -- TLS1.2 only from hereon in --
        self.cross_check_certificate_and_save_sni(sess, sni.clone(), &certkey)?;
        self.handshake.transcript.add_message(&m);

        // Save their Random.
        client_hello.random.write_slice(&mut self.handshake.randoms.client);

        if client_hello.ems_support_offered() {
            self.handshake.using_ems = true;
        }

        let groups_ext = client_hello.get_namedgroups_extension()
            .ok_or_else(|| incompatible(sess, "client didn't describe groups"))?;
        let ecpoints_ext = client_hello.get_ecpoints_extension()
            .ok_or_else(|| incompatible(sess, "client didn't describe ec points"))?;

        debug!("namedgroups {:?}", groups_ext);
        debug!("ecpoints {:?}", ecpoints_ext);

        if !ecpoints_ext.contains(&ECPointFormat::Uncompressed) {
            sess.common.send_fatal_alert(AlertDescription::IllegalParameter);
            return Err(TLSError::PeerIncompatibleError("client didn't support uncompressed ec points"
                .to_string()));
        }

        // -- Check for resumption --
        // We can do this either by (in order of preference):
        // 1. receiving a ticket that decrypts
        // 2. receiving a sessionid that is in our cache
        //
        // If we receive a ticket, the sessionid won't be in our
        // cache, so don't check.
        //
        // If either works, we end up with a ServerSessionValue
        // which is passed to start_resumption and concludes
        // our handling of the ClientHello.
        //
        let mut ticket_received = false;

        if let Some(ticket_ext) = client_hello.get_ticket_extension() {
            if let ClientExtension::SessionTicketOffer(ref ticket) = *ticket_ext {
                ticket_received = true;
                info!("Ticket received");

                let maybe_resume = sess.config
                    .ticketer
                    .decrypt(&ticket.0)
                    .and_then(|plain| persist::ServerSessionValue::read_bytes(&plain));

                if can_resume(sess, &self.handshake, &maybe_resume) {
                    return self.start_resumption(sess,
                                                 client_hello, sni.as_ref(),
                                                 &client_hello.session_id,
                                                 maybe_resume.unwrap());
                } else {
                    info!("Ticket didn't decrypt");
                }
            }
        }

        // Perhaps resume?  If we received a ticket, the sessionid
        // does not correspond to a real session.
        if !client_hello.session_id.is_empty() && !ticket_received {
            let maybe_resume = sess.config.session_storage
                .get(&client_hello.session_id)
                .and_then(|x| persist::ServerSessionValue::read_bytes(&x));

            if can_resume(sess, &self.handshake, &maybe_resume) {
                return self.start_resumption(sess,
                                             client_hello, sni.as_ref(),
                                             &client_hello.session_id,
                                             maybe_resume.unwrap());
            }
        }

        // Now we have chosen a ciphersuite, we can make kx decisions.
        let sigscheme = sess.common.get_suite()
            .resolve_sig_scheme(sigschemes_ext)
            .ok_or_else(|| incompatible(sess, "no supported sig scheme"))?;

        let group = util::first_in_both(NamedGroups::supported().as_slice(),
                                        groups_ext.as_slice())
            .ok_or_else(|| incompatible(sess, "no supported group"))?;

        let ecpoint = util::first_in_both(ECPointFormatList::supported().as_slice(),
                                          ecpoints_ext.as_slice())
            .ok_or_else(|| incompatible(sess, "no supported point format"))?;

        debug_assert_eq!(ecpoint, ECPointFormat::Uncompressed);

        self.emit_server_hello(sess, Some(&mut certkey), client_hello, false)?;
        self.emit_certificate(sess, &mut certkey);
        self.emit_cert_status(sess, &mut certkey);
        let kx = self.emit_server_kx(sess, sigscheme, &group, &mut certkey)?;
        let doing_client_auth = self.emit_certificate_req(sess);
        self.emit_server_hello_done(sess);

        if doing_client_auth {
            Ok(self.into_expect_tls12_certificate(kx))
        } else {
            Ok(self.into_expect_tls12_client_kx(kx))
        }
    }
}

// --- Process client's Certificate for client auth ---
pub struct ExpectTLS12Certificate {
    handshake: HandshakeDetails,
    server_kx: ServerKXDetails,
    send_ticket: bool,
}

impl ExpectTLS12Certificate {
    fn into_expect_tls12_client_kx(self, cert: Option<ClientCertDetails>) -> Box<State + Send> {
        Box::new(ExpectTLS12ClientKX {
            handshake: self.handshake,
            server_kx: self.server_kx,
            client_cert: cert,
            send_ticket: self.send_ticket,
        })
    }
}

impl State for ExpectTLS12Certificate {
    fn check_message(&self, m: &Message) -> CheckResult {
        check_handshake_message(m, &[HandshakeType::Certificate])
    }

    fn handle(mut self: Box<Self>, sess: &mut ServerSessionImpl, m: Message) -> StateResult {
        let cert_chain = extract_handshake!(m, HandshakePayload::Certificate).unwrap();
        self.handshake.transcript.add_message(&m);

        if cert_chain.is_empty() && !sess.config.client_auth_mandatory {
            info!("client auth requested but no certificate supplied");
            self.handshake.transcript.abandon_client_auth();
            return Ok(self.into_expect_tls12_client_kx(None));
        }

        debug!("certs {:?}", cert_chain);

        sess.config.get_verifier().verify_client_cert(&sess.config.client_auth_roots,
                                                      cert_chain)
            .or_else(|err| {
                     incompatible(sess, "certificate invalid");
                     Err(err)
                     })?;

        let cert = ClientCertDetails::new(cert_chain.clone());
        Ok(self.into_expect_tls12_client_kx(Some(cert)))
    }
}

pub struct ExpectTLS13Certificate {
    handshake: HandshakeDetails,
    send_ticket: bool,
}

impl ExpectTLS13Certificate {
    fn into_expect_tls13_finished(self) -> Box<State + Send> {
        Box::new(ExpectTLS13Finished {
            handshake: self.handshake,
            send_ticket: self.send_ticket,
        })
    }

    fn into_expect_tls13_certificate_verify(self,
                                            cert: ClientCertDetails) -> Box<State + Send> {
        Box::new(ExpectTLS13CertificateVerify {
            handshake: self.handshake,
            client_cert: cert,
            send_ticket: self.send_ticket,
        })
    }
}

impl State for ExpectTLS13Certificate {
    fn check_message(&self, m: &Message) -> CheckResult {
        check_handshake_message(m, &[HandshakeType::Certificate])
    }

    fn handle(mut self: Box<Self>, sess: &mut ServerSessionImpl, m: Message) -> StateResult {
        let certp = extract_handshake!(m, HandshakePayload::CertificateTLS13).unwrap();
        self.handshake.transcript.add_message(&m);

        // We don't send any CertificateRequest extensions, so any extensions
        // here are illegal.
        if certp.any_entry_has_extension() {
            return Err(TLSError::PeerMisbehavedError("client sent unsolicited cert extension"
                                                     .to_string()));
        }

        let cert_chain = certp.convert();

        if cert_chain.is_empty() {
            if !sess.config.client_auth_mandatory {
                info!("client auth requested but no certificate supplied");
                self.handshake.transcript.abandon_client_auth();
                return Ok(self.into_expect_tls13_finished());
            }

            sess.common.send_fatal_alert(AlertDescription::CertificateRequired);
            return Err(TLSError::NoCertificatesPresented);
        }

        sess.config.get_verifier().verify_client_cert(&sess.config.client_auth_roots,
                                                      &cert_chain)?;

        let cert = ClientCertDetails::new(cert_chain);
        Ok(self.into_expect_tls13_certificate_verify(cert))
    }
}

// --- Process client's KeyExchange ---
pub struct ExpectTLS12ClientKX {
    handshake: HandshakeDetails,
    server_kx: ServerKXDetails,
    client_cert: Option<ClientCertDetails>,
    send_ticket: bool,
}

impl ExpectTLS12ClientKX {
    fn into_expect_tls12_certificate_verify(self) -> Box<State + Send> {
        Box::new(ExpectTLS12CertificateVerify {
            handshake: self.handshake,
            client_cert: self.client_cert.unwrap(),
            send_ticket: self.send_ticket,
        })
    }

    fn into_expect_tls12_ccs(self) -> Box<State + Send> {
        Box::new(ExpectTLS12CCS {
            handshake: self.handshake,
            resuming: false,
            send_ticket: self.send_ticket,
        })
    }
}

impl State for ExpectTLS12ClientKX {
    fn check_message(&self, m: &Message) -> CheckResult {
        check_handshake_message(m, &[HandshakeType::ClientKeyExchange])
    }

    fn handle(mut self: Box<Self>, sess: &mut ServerSessionImpl, m: Message) -> StateResult {
        let client_kx = extract_handshake!(m, HandshakePayload::ClientKeyExchange).unwrap();
        self.handshake.transcript.add_message(&m);

        // Complete key agreement, and set up encryption with the
        // resulting premaster secret.
        let kx = self.server_kx.take_kx();
        if !kx.check_client_params(&client_kx.0) {
            sess.common.send_fatal_alert(AlertDescription::DecodeError);
            return Err(TLSError::CorruptMessagePayload(ContentType::Handshake));
        }

        let kxd = kx.server_complete(&client_kx.0)
            .ok_or_else(|| TLSError::PeerMisbehavedError("key exchange completion failed"
                                                         .to_string()))?;

        let hashalg = sess.common.get_suite().get_hash();
        if self.handshake.using_ems {
            let handshake_hash = self.handshake.transcript.get_current_hash();
            sess.secrets = Some(SessionSecrets::new_ems(&self.handshake.randoms,
                                                        &handshake_hash,
                                                        hashalg,
                                                        &kxd.premaster_secret));
        } else {
            sess.secrets = Some(SessionSecrets::new(&self.handshake.randoms,
                                                    hashalg,
                                                    &kxd.premaster_secret));
        }
        sess.start_encryption_tls12();

        if self.client_cert.is_some() {
            Ok(self.into_expect_tls12_certificate_verify())
        } else {
            Ok(self.into_expect_tls12_ccs())
        }
    }
}

// --- Process client's certificate proof ---
pub struct ExpectTLS12CertificateVerify {
    handshake: HandshakeDetails,
    client_cert: ClientCertDetails,
    send_ticket: bool,
}

impl ExpectTLS12CertificateVerify {
    fn into_expect_tls12_ccs(self) -> Box<State + Send> {
        Box::new(ExpectTLS12CCS {
            handshake: self.handshake,
            resuming: false,
            send_ticket: self.send_ticket,
        })
    }
}

impl State for ExpectTLS12CertificateVerify {
    fn check_message(&self, m: &Message) -> CheckResult {
        check_handshake_message(m, &[HandshakeType::CertificateVerify])
    }

    fn handle(mut self: Box<Self>, sess: &mut ServerSessionImpl, m: Message) -> StateResult {
        let rc = {
            let sig = extract_handshake!(m, HandshakePayload::CertificateVerify).unwrap();
            let handshake_msgs = self.handshake.transcript.take_handshake_buf();
            let certs = &self.client_cert.cert_chain;

            verify::verify_signed_struct(&handshake_msgs, &certs[0], sig)
        };

        if let Err(e) = rc {
            sess.common.send_fatal_alert(AlertDescription::AccessDenied);
            return Err(e);
        } else {
            debug!("client CertificateVerify OK");
            sess.client_cert_chain = Some(self.client_cert.take_chain());
        }

        self.handshake.transcript.add_message(&m);
        Ok(self.into_expect_tls12_ccs())
    }
}

pub struct ExpectTLS13CertificateVerify {
    handshake: HandshakeDetails,
    client_cert: ClientCertDetails,
    send_ticket: bool,
}

impl ExpectTLS13CertificateVerify {
    fn into_expect_tls13_finished(self) -> Box<State + Send> {
        Box::new(ExpectTLS13Finished {
            handshake: self.handshake,
            send_ticket: self.send_ticket,
        })
    }
}

impl State for ExpectTLS13CertificateVerify {
    fn check_message(&self, m: &Message) -> CheckResult {
        check_handshake_message(m, &[HandshakeType::CertificateVerify])
    }

    fn handle(mut self: Box<Self>, sess: &mut ServerSessionImpl, m: Message) -> StateResult {
        let rc = {
            let sig = extract_handshake!(m, HandshakePayload::CertificateVerify).unwrap();
            let handshake_hash = self.handshake.transcript.get_current_hash();
            self.handshake.transcript.abandon_client_auth();
            let certs = &self.client_cert.cert_chain;

            verify::verify_tls13(&certs[0],
                                 sig,
                                 &handshake_hash,
                                 b"TLS 1.3, client CertificateVerify\x00")
        };

        if let Err(e) = rc {
            sess.common.send_fatal_alert(AlertDescription::AccessDenied);
            return Err(e);
        } else {
            debug!("client CertificateVerify OK");
            sess.client_cert_chain = Some(self.client_cert.take_chain());
        }

        self.handshake.transcript.add_message(&m);
        Ok(self.into_expect_tls13_finished())
    }
}

// --- Process client's ChangeCipherSpec ---
pub struct ExpectTLS12CCS {
    handshake: HandshakeDetails,
    resuming: bool,
    send_ticket: bool,
}

impl ExpectTLS12CCS {
    fn into_expect_tls12_finished(self) -> Box<State + Send> {
        Box::new(ExpectTLS12Finished {
            handshake: self.handshake,
            resuming: self.resuming,
            send_ticket: self.send_ticket,
        })
    }
}

impl State for ExpectTLS12CCS {
    fn check_message(&self, m: &Message) -> CheckResult {
        check_message(m, &[ContentType::ChangeCipherSpec], &[])
    }

    fn handle(self: Box<Self>, sess: &mut ServerSessionImpl, _m: Message) -> StateResult {
        // CCS should not be received interleaved with fragmented handshake-level
        // message.
        if !sess.common.handshake_joiner.is_empty() {
            warn!("CCS received interleaved with fragmented handshake");
            return Err(TLSError::InappropriateMessage {
                expect_types: vec![ ContentType::Handshake ],
                got_type: ContentType::ChangeCipherSpec,
            });
        }

        sess.common.peer_now_encrypting();
        Ok(self.into_expect_tls12_finished())
    }
}

// --- Process client's Finished ---
fn get_server_session_value(handshake: &HandshakeDetails,
                            sess: &ServerSessionImpl) -> persist::ServerSessionValue {
    let scs = sess.common.get_suite();

    let (version, secret) = if sess.common.is_tls13() {
        let handshake_hash = handshake
            .transcript
            .get_current_hash();
        let resume_secret = sess.common
            .get_key_schedule()
            .derive(SecretKind::ResumptionMasterSecret, &handshake_hash);
        (ProtocolVersion::TLSv1_3, resume_secret)
    } else {
        (ProtocolVersion::TLSv1_2, sess.secrets.as_ref().unwrap().get_master_secret())
    };

    let mut v = persist::ServerSessionValue::new(sess.get_sni(), version,
                                                 scs.suite, secret,
                                                 &sess.client_cert_chain);

    if handshake.using_ems {
        v.set_extended_ms_used();
    }

    v
}

fn emit_ticket(handshake: &mut HandshakeDetails,
               sess: &mut ServerSessionImpl) {
    // If we can't produce a ticket for some reason, we can't
    // report an error. Send an empty one.
    let plain = get_server_session_value(handshake, sess)
        .get_encoding();
    let ticket = sess.config
        .ticketer
        .encrypt(&plain)
        .unwrap_or_else(Vec::new);
    let ticket_lifetime = sess.config.ticketer.get_lifetime();

    let m = Message {
        typ: ContentType::Handshake,
        version: ProtocolVersion::TLSv1_2,
        payload: MessagePayload::Handshake(HandshakeMessagePayload {
            typ: HandshakeType::NewSessionTicket,
            payload:
                HandshakePayload::NewSessionTicket(NewSessionTicketPayload::new(ticket_lifetime,
                                                                                ticket)),
        }),
    };

    handshake.transcript.add_message(&m);
    sess.common.send_msg(m, false);
}

fn emit_ccs(sess: &mut ServerSessionImpl) {
    let m = Message {
        typ: ContentType::ChangeCipherSpec,
        version: ProtocolVersion::TLSv1_2,
        payload: MessagePayload::ChangeCipherSpec(ChangeCipherSpecPayload {}),
    };

    sess.common.send_msg(m, false);
    sess.common.we_now_encrypting();
}

fn emit_finished(handshake: &mut HandshakeDetails, sess: &mut ServerSessionImpl) {
    let vh = handshake.transcript.get_current_hash();
    let verify_data = sess.secrets.as_ref().unwrap().server_verify_data(&vh);
    let verify_data_payload = Payload::new(verify_data);

    let f = Message {
        typ: ContentType::Handshake,
        version: ProtocolVersion::TLSv1_2,
        payload: MessagePayload::Handshake(HandshakeMessagePayload {
            typ: HandshakeType::Finished,
            payload: HandshakePayload::Finished(verify_data_payload),
        }),
    };

    handshake.transcript.add_message(&f);
    sess.common.send_msg(f, true);
}

pub struct ExpectTLS12Finished {
    handshake: HandshakeDetails,
    resuming: bool,
    send_ticket: bool,
}

impl ExpectTLS12Finished {
    fn into_expect_tls12_traffic(self, fin: verify::FinishedMessageVerified) -> Box<State + Send> {
        Box::new(ExpectTLS12Traffic {
            _fin_verified: fin,
        })
    }
}

impl State for ExpectTLS12Finished {
    fn check_message(&self, m: &Message) -> CheckResult {
        check_handshake_message(m, &[HandshakeType::Finished])
    }

    fn handle(mut self: Box<Self>, sess: &mut ServerSessionImpl, m: Message) -> StateResult {
        let finished = extract_handshake!(m, HandshakePayload::Finished).unwrap();

        let vh = self.handshake.transcript.get_current_hash();
        let expect_verify_data = sess.secrets.as_ref().unwrap().client_verify_data(&vh);

        let fin = constant_time::verify_slices_are_equal(&expect_verify_data, &finished.0)
            .map_err(|_| {
                     sess.common.send_fatal_alert(AlertDescription::DecryptError);
                     TLSError::DecryptError
                     })
            .map(|_| verify::FinishedMessageVerified::assertion())?;

        // Save session, perhaps
        if !self.resuming && !self.handshake.session_id.is_empty() {
            let value = get_server_session_value(&self.handshake, sess);

            let worked = sess.config.session_storage
                .put(&self.handshake.session_id, value.get_encoding());
            if worked {
                info!("Session saved");
            } else {
                info!("Session not saved");
            }
        }

        // Send our CCS and Finished.
        self.handshake.transcript.add_message(&m);
        if !self.resuming {
            if self.send_ticket {
                emit_ticket(&mut self.handshake,
                            sess);
            }
            emit_ccs(sess);
            emit_finished(&mut self.handshake,
                          sess);
        }

        sess.common.we_now_encrypting();
        sess.common.start_traffic();
        Ok(self.into_expect_tls12_traffic(fin))
    }
}

pub struct ExpectTLS13Finished {
    handshake: HandshakeDetails,
    send_ticket: bool,
}

impl ExpectTLS13Finished {
    fn into_expect_tls13_traffic(self, fin: verify::FinishedMessageVerified) -> Box<State + Send> {
        Box::new(ExpectTLS13Traffic {
            _fin_verified: fin,
        })
    }

    fn emit_ticket_tls13(&mut self, sess: &mut ServerSessionImpl) {
        if !self.send_ticket {
            return;
        }

        let plain = get_server_session_value(&self.handshake, sess)
            .get_encoding();
        let maybe_ticket = sess.config
            .ticketer
            .encrypt(&plain);
        let ticket_lifetime = sess.config.ticketer.get_lifetime();

        if maybe_ticket.is_none() {
            return;
        }

        let ticket = maybe_ticket.unwrap();
        let age_add = rand::random_u32(); // nb, we don't do 0-RTT data, so whatever
        let payload = NewSessionTicketPayloadTLS13::new(ticket_lifetime, age_add, ticket);
        let m = Message {
            typ: ContentType::Handshake,
            version: ProtocolVersion::TLSv1_3,
            payload: MessagePayload::Handshake(HandshakeMessagePayload {
                typ: HandshakeType::NewSessionTicket,
                payload: HandshakePayload::NewSessionTicketTLS13(payload),
            }),
        };

        debug!("sending new ticket {:?}", m);
        self.handshake.transcript.add_message(&m);
        sess.common.send_msg(m, true);
    }
}

impl State for ExpectTLS13Finished {
    fn check_message(&self, m: &Message) -> CheckResult {
        check_handshake_message(m, &[HandshakeType::Finished])
    }

    fn handle(mut self: Box<Self>, sess: &mut ServerSessionImpl, m: Message) -> StateResult {
        let finished = extract_handshake!(m, HandshakePayload::Finished).unwrap();

        let handshake_hash = self.handshake.transcript.get_current_hash();
        let expect_verify_data = sess.common
            .get_key_schedule()
            .sign_finish(SecretKind::ClientHandshakeTrafficSecret, &handshake_hash);

        let fin = constant_time::verify_slices_are_equal(&expect_verify_data, &finished.0)
            .map_err(|_| {
                     sess.common.send_fatal_alert(AlertDescription::DecryptError);
                     warn!("Finished wrong");
                     TLSError::DecryptError
                     })
            .map(|_| verify::FinishedMessageVerified::assertion())?;

        // nb. future derivations include Client Finished, but not the
        // main application data keying.
        self.handshake.transcript.add_message(&m);

        // Now move to using application data keys for client traffic.
        // Server traffic is already done.
        let read_key = sess.common
            .get_key_schedule()
            .derive(SecretKind::ClientApplicationTrafficSecret,
                    &self.handshake.hash_at_server_fin);

        let suite = sess.common.get_suite();
        check_aligned_handshake(sess)?;
        sess.common.set_message_decrypter(cipher::new_tls13_read(suite, &read_key));
        sess.common
            .get_mut_key_schedule()
            .current_client_traffic_secret = read_key;

        if sess.config.ticketer.enabled() {
            self.emit_ticket_tls13(sess);
        }

        sess.common.we_now_encrypting();
        sess.common.start_traffic();
        Ok(self.into_expect_tls13_traffic(fin))
    }
}

// --- Process traffic ---
pub struct ExpectTLS12Traffic {
    _fin_verified: verify::FinishedMessageVerified,
}

impl ExpectTLS12Traffic {
}

impl State for ExpectTLS12Traffic {
    fn check_message(&self, m: &Message) -> CheckResult {
        check_message(m, &[ContentType::ApplicationData], &[])
    }

    fn handle(self: Box<Self>, sess: &mut ServerSessionImpl, mut m: Message) -> StateResult {
        sess.common.take_received_plaintext(m.take_opaque_payload().unwrap());
        Ok(self)
    }
}

pub struct ExpectTLS13Traffic {
    _fin_verified: verify::FinishedMessageVerified,
}

impl ExpectTLS13Traffic {
    fn handle_traffic(&self, sess: &mut ServerSessionImpl, mut m: Message) -> Result<(), TLSError> {
        sess.common.take_received_plaintext(m.take_opaque_payload().unwrap());
        Ok(())
    }

    fn handle_key_update(&self, sess: &mut ServerSessionImpl, m: Message) -> Result<(), TLSError> {
        let kur = extract_handshake!(m, HandshakePayload::KeyUpdate).unwrap();
        sess.common.process_key_update(kur, SecretKind::ClientApplicationTrafficSecret)
    }
}

impl State for ExpectTLS13Traffic {
    fn check_message(&self, m: &Message) -> CheckResult {
        check_message(m,
                      &[ContentType::ApplicationData, ContentType::Handshake],
                      &[HandshakeType::KeyUpdate])
    }

    fn handle(self: Box<Self>, sess: &mut ServerSessionImpl, m: Message) -> StateResult {
        if m.is_content_type(ContentType::ApplicationData) {
            self.handle_traffic(sess, m)?;
        } else if m.is_handshake_type(HandshakeType::KeyUpdate) {
            self.handle_key_update(sess, m)?;
        }

        Ok(self)
    }
}

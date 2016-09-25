#[macro_use]
mod macros;

pub mod codec;
pub mod base;
#[allow(non_camel_case_types)]
pub mod enums;
pub mod alert;
#[allow(non_camel_case_types)]
pub mod handshake;
pub mod ccs;
pub mod message;
pub mod persist;
pub mod deframer;
pub mod fragmenter;
pub mod hsjoiner;

#[cfg(test)]
mod enums_test;

#[cfg(test)]
mod test {
  #[test]
  fn smoketest() {
    use super::codec::Reader;
    use super::message::Message;
    use super::codec::Codec;
    let mut r = Reader::init(b"\x16\x03\x01\x01\x49\x01\x00\x01\x45\x03\x03\x37\x84\xff\xb8\x8d\xeb\x79\xcc\x8c\xb8\xd4\x7e\xf7\x99\x75\x1e\x60\x30\x9a\x18\xf9\x90\xa9\xae\x60\x6c\xf7\xa5\xf8\x95\x88\xf6\x00\x00\xb4\xc0\x30\xc0\x2c\xc0\x28\xc0\x24\xc0\x14\xc0\x0a\x00\xa5\x00\xa3\x00\xa1\x00\x9f\x00\x6b\x00\x6a\x00\x69\x00\x68\x00\x39\x00\x38\x00\x37\x00\x36\x00\x88\x00\x87\x00\x86\x00\x85\xc0\x32\xc0\x2e\xc0\x2a\xc0\x26\xc0\x0f\xc0\x05\x00\x9d\x00\x3d\x00\x35\x00\x84\xc0\x2f\xc0\x2b\xc0\x27\xc0\x23\xc0\x13\xc0\x09\x00\xa4\x00\xa2\x00\xa0\x00\x9e\x00\x67\x00\x40\x00\x3f\x00\x3e\x00\x33\x00\x32\x00\x31\x00\x30\x00\x9a\x00\x99\x00\x98\x00\x97\x00\x45\x00\x44\x00\x43\x00\x42\xc0\x31\xc0\x2d\xc0\x29\xc0\x25\xc0\x0e\xc0\x04\x00\x9c\x00\x3c\x00\x2f\x00\x96\x00\x41\xc0\x11\xc0\x07\xc0\x0c\xc0\x02\x00\x05\x00\x04\xc0\x12\xc0\x08\x00\x16\x00\x13\x00\x10\x00\x0d\xc0\x0d\xc0\x03\x00\x0a\x00\x15\x00\x12\x00\x0f\x00\x0c\x00\x09\x00\xff\x01\x00\x00\x68\x00\x00\x00\x0f\x00\x0d\x00\x00\x0a\x67\x6f\x6f\x67\x6c\x65\x2e\x63\x6f\x6d\x00\x0b\x00\x04\x03\x00\x01\x02\x00\x0a\x00\x1c\x00\x1a\x00\x17\x00\x19\x00\x1c\x00\x1b\x00\x18\x00\x1a\x00\x16\x00\x0e\x00\x0d\x00\x0b\x00\x0c\x00\x09\x00\x0a\x00\x23\x00\x00\x00\x0d\x00\x20\x00\x1e\x06\x01\x06\x02\x06\x03\x05\x01\x05\x02\x05\x03\x04\x01\x04\x02\x04\x03\x03\x01\x03\x02\x03\x03\x02\x01\x02\x02\x02\x03\x00\x0f\x00\x01\x01\x16\x03\x03\x00\x3f\x02\x00\x00\x3b\x03\x03\x57\x29\x07\x7f\x43\xa5\xdd\xb3\x18\xdc\x74\x37\xc2\x0f\x77\x0e\x73\xd6\xa5\x96\x79\x24\x51\x4a\xa3\xe7\xcd\x44\x65\xce\x7c\xba\x00\xc0\x2b\x00\x00\x13\xff\x01\x00\x01\x00\x00\x00\x00\x00\x00\x23\x00\x00\x00\x0b\x00\x02\x01\x00\x16\x03\x03\x0e\x80\x0b\x00\x0e\x7c\x00\x0e\x79\x00\x06\xfb\x30\x82\x06\xf7\x30\x82\x05\xdf\xa0\x03\x02\x01\x02\x02\x08\x1b\x7c\x9d\xb9\xbe\xf4\x92\xd2\x30\x0d\x06\x09\x2a\x86\x48\x86\xf7\x0d\x01\x01\x0b\x05\x00\x30\x49\x31\x0b\x30\x09\x06\x03\x55\x04\x06\x13\x02\x55\x53\x31\x13\x30\x11\x06\x03\x55\x04\x0a\x13\x0a\x47\x6f\x6f\x67\x6c\x65\x20\x49\x6e\x63\x31\x25\x30\x23\x06\x03\x55\x04\x03\x13\x1c\x47\x6f\x6f\x67\x6c\x65\x20\x49\x6e\x74\x65\x72\x6e\x65\x74\x20\x41\x75\x74\x68\x6f\x72\x69\x74\x79\x20\x47\x32\x30\x1e\x17\x0d\x31\x36\x30\x34\x32\x30\x31\x33\x35\x37\x34\x31\x5a\x17\x0d\x31\x36\x30\x37\x31\x33\x31\x33\x30\x38\x30\x30\x5a\x30\x66\x31\x0b\x30\x09\x06\x03\x55\x04\x06\x13\x02\x55\x53\x31\x13\x30\x11\x06\x03\x55\x04\x08\x0c\x0a\x43\x61\x6c\x69\x66\x6f\x72\x6e\x69\x61\x31\x16\x30\x14\x06\x03\x55\x04\x07\x0c\x0d\x4d\x6f\x75\x6e\x74\x61\x69\x6e\x20\x56\x69\x65\x77\x31\x13\x30\x11\x06\x03\x55\x04\x0a\x0c\x0a\x47\x6f\x6f\x67\x6c\x65\x20\x49\x6e\x63\x31\x15\x30\x13\x06\x03\x55\x04\x03\x0c\x0c\x2a\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x63\x6f\x6d\x30\x59\x30\x13\x06\x07\x2a\x86\x48\xce\x3d\x02\x01\x06\x08\x2a\x86\x48\xce\x3d\x03\x01\x07\x03\x42\x00\x04\x39\x11\xed\x86\xe0\x4a\xd1\x50\x08\x8d\x6a\x56\x89\x1f\x83\x97\xf4\x65\x83\xc8\x3d\x42\xb9\x30\x90\x21\x9e\x90\x13\x9d\xec\x5f\xea\xea\x79\xac\xdd\xd3\xe2\xc4\x77\x1f\x91\x30\x78\x22\xb1\x62\x5b\xc8\xb2\x85\x7e\x21\xb7\x33\xf6\xc6\x2e\x33\xcc\xe9\xde\xed\xa3\x82\x04\x8f\x30\x82\x04\x8b\x30\x1d\x06\x03\x55\x1d\x25\x04\x16\x30\x14\x06\x08\x2b\x06\x01\x05\x05\x07\x03\x01\x06\x08\x2b\x06\x01\x05\x05\x07\x03\x02\x30\x82\x03\x4e\x06\x03\x55\x1d\x11\x04\x82\x03\x45\x30\x82\x03\x41\x82\x0c\x2a\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x63\x6f\x6d\x82\x0d\x2a\x2e\x61\x6e\x64\x72\x6f\x69\x64\x2e\x63\x6f\x6d\x82\x16\x2a\x2e\x61\x70\x70\x65\x6e\x67\x69\x6e\x65\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x63\x6f\x6d\x82\x12\x2a\x2e\x63\x6c\x6f\x75\x64\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x63\x6f\x6d\x82\x16\x2a\x2e\x67\x6f\x6f\x67\x6c\x65\x2d\x61\x6e\x61\x6c\x79\x74\x69\x63\x73\x2e\x63\x6f\x6d\x82\x0b\x2a\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x63\x61\x82\x0b\x2a\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x63\x6c\x82\x0e\x2a\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x63\x6f\x2e\x69\x6e\x82\x0e\x2a\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x63\x6f\x2e\x6a\x70\x82\x0e\x2a\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x63\x6f\x2e\x75\x6b\x82\x0f\x2a\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x63\x6f\x6d\x2e\x61\x72\x82\x0f\x2a\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x63\x6f\x6d\x2e\x61\x75\x82\x0f\x2a\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x63\x6f\x6d\x2e\x62\x72\x82\x0f\x2a\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x63\x6f\x6d\x2e\x63\x6f\x82\x0f\x2a\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x63\x6f\x6d\x2e\x6d\x78\x82\x0f\x2a\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x63\x6f\x6d\x2e\x74\x72\x82\x0f\x2a\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x63\x6f\x6d\x2e\x76\x6e\x82\x0b\x2a\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x64\x65\x82\x0b\x2a\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x65\x73\x82\x0b\x2a\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x66\x72\x82\x0b\x2a\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x68\x75\x82\x0b\x2a\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x69\x74\x82\x0b\x2a\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x6e\x6c\x82\x0b\x2a\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x70\x6c\x82\x0b\x2a\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x70\x74\x82\x12\x2a\x2e\x67\x6f\x6f\x67\x6c\x65\x61\x64\x61\x70\x69\x73\x2e\x63\x6f\x6d\x82\x0f\x2a\x2e\x67\x6f\x6f\x67\x6c\x65\x61\x70\x69\x73\x2e\x63\x6e\x82\x14\x2a\x2e\x67\x6f\x6f\x67\x6c\x65\x63\x6f\x6d\x6d\x65\x72\x63\x65\x2e\x63\x6f\x6d\x82\x11\x2a\x2e\x67\x6f\x6f\x67\x6c\x65\x76\x69\x64\x65\x6f\x2e\x63\x6f\x6d\x82\x0c\x2a\x2e\x67\x73\x74\x61\x74\x69\x63\x2e\x63\x6e\x82\x0d\x2a\x2e\x67\x73\x74\x61\x74\x69\x63\x2e\x63\x6f\x6d\x82\x0a\x2a\x2e\x67\x76\x74\x31\x2e\x63\x6f\x6d\x82\x0a\x2a\x2e\x67\x76\x74\x32\x2e\x63\x6f\x6d\x82\x14\x2a\x2e\x6d\x65\x74\x72\x69\x63\x2e\x67\x73\x74\x61\x74\x69\x63\x2e\x63\x6f\x6d\x82\x0c\x2a\x2e\x75\x72\x63\x68\x69\x6e\x2e\x63\x6f\x6d\x82\x10\x2a\x2e\x75\x72\x6c\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x63\x6f\x6d\x82\x16\x2a\x2e\x79\x6f\x75\x74\x75\x62\x65\x2d\x6e\x6f\x63\x6f\x6f\x6b\x69\x65\x2e\x63\x6f\x6d\x82\x0d\x2a\x2e\x79\x6f\x75\x74\x75\x62\x65\x2e\x63\x6f\x6d\x82\x16\x2a\x2e\x79\x6f\x75\x74\x75\x62\x65\x65\x64\x75\x63\x61\x74\x69\x6f\x6e\x2e\x63\x6f\x6d\x82\x0b\x2a\x2e\x79\x74\x69\x6d\x67\x2e\x63\x6f\x6d\x82\x1a\x61\x6e\x64\x72\x6f\x69\x64\x2e\x63\x6c\x69\x65\x6e\x74\x73\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x63\x6f\x6d\x82\x0b\x61\x6e\x64\x72\x6f\x69\x64\x2e\x63\x6f\x6d\x82\x04\x67\x2e\x63\x6f\x82\x06\x67\x6f\x6f\x2e\x67\x6c\x82\x14\x67\x6f\x6f\x67\x6c\x65\x2d\x61\x6e\x61\x6c\x79\x74\x69\x63\x73\x2e\x63\x6f\x6d\x82\x0a\x67\x6f\x6f\x67\x6c\x65\x2e\x63\x6f\x6d\x82\x12\x67\x6f\x6f\x67\x6c\x65\x63\x6f\x6d\x6d\x65\x72\x63\x65\x2e\x63\x6f\x6d\x82\x0a\x75\x72\x63\x68\x69\x6e\x2e\x63\x6f\x6d\x82\x0a\x77\x77\x77\x2e\x67\x6f\x6f\x2e\x67\x6c\x82\x08\x79\x6f\x75\x74\x75\x2e\x62\x65\x82\x0b\x79\x6f\x75\x74\x75\x62\x65\x2e\x63\x6f\x6d\x82\x14\x79\x6f\x75\x74\x75\x62\x65\x65\x64\x75\x63\x61\x74\x69\x6f\x6e\x2e\x63\x6f\x6d\x30\x0b\x06\x03\x55\x1d\x0f\x04\x04\x03\x02\x07\x80\x30\x68\x06\x08\x2b\x06\x01\x05\x05\x07\x01\x01\x04\x5c\x30\x5a\x30\x2b\x06\x08\x2b\x06\x01\x05\x05\x07\x30\x02\x86\x1f\x68\x74\x74\x70\x3a\x2f\x2f\x70\x6b\x69\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x63\x6f\x6d\x2f\x47\x49\x41\x47\x32\x2e\x63\x72\x74\x30\x2b\x06\x08\x2b\x06\x01\x05\x05\x07\x30\x01\x86\x1f\x68\x74\x74\x70\x3a\x2f\x2f\x63\x6c\x69\x65\x6e\x74\x73\x31\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x63\x6f\x6d\x2f\x6f\x63\x73\x70\x30\x1d\x06\x03\x55\x1d\x0e\x04\x16\x04\x14\xc8\x25\x22\xba\x8a\x04\x37\x07\xd9\x9e\x62\xb6\x6f\x1d\xe9\x16\x77\x60\xcd\xa2\x30\x0c\x06\x03\x55\x1d\x13\x01\x01\xff\x04\x02\x30\x00\x30\x1f\x06\x03\x55\x1d\x23\x04\x18\x30\x16\x80\x14\x4a\xdd\x06\x16\x1b\xbc\xf6\x68\xb5\x76\xf5\x81\xb6\xbb\x62\x1a\xba\x5a\x81\x2f\x30\x21\x06\x03\x55\x1d\x20\x04\x1a\x30\x18\x30\x0c\x06\x0a\x2b\x06\x01\x04\x01\xd6\x79\x02\x05\x01\x30\x08\x06\x06\x67\x81\x0c\x01\x02\x02\x30\x30\x06\x03\x55\x1d\x1f\x04\x29\x30\x27\x30\x25\xa0\x23\xa0\x21\x86\x1f\x68\x74\x74\x70\x3a\x2f\x2f\x70\x6b\x69\x2e\x67\x6f\x6f\x67\x6c\x65\x2e\x63\x6f\x6d\x2f\x47\x49\x41\x47\x32\x2e\x63\x72\x6c\x30\x0d\x06\x09\x2a\x86\x48\x86\xf7\x0d\x01\x01\x0b\x05\x00\x03\x82\x01\x01\x00\x72\xf1\x41\x65\xea\x39\x3c\xb1\xbf\x96\x7a\x1d\xb4\x9d\x29\xc2\x08\x2d\x1f\xef\x0c\x97\x23\x53\x4b\xff\x4f\x54\xbb\x30\x3a\x64\xdd\x52\xa3\x25\xb9\xc0\x1b\xf6\xb4\x82\x36\xd8\xde\x90\x81\x46\xac\xe8\xd3\x08\xe2\x7f\x03\xff\x72\x1e\x58\x54\xdd\x62\x4d\x30\xcc\xed\x7c\x96\x71\x2a\xb1\x83\x20\x11\x0d\x9a\xa7\x24\x1b\x80\x8b\xba\xf4\x53\x8e\xe2\x77\x82\x0f\x5d\x81\xb1\x85\x70\xd5\xb2\x8f\x7e\x51\xd2\xf1\x08\x49\x1a\xf1\x7a\xe2\xdd\x67\x97\x57\xa6\x5a\x26\x6c\x58\xf4\x9e\x46\x76\xac\xb7\xe6\xfa\xb3\x04\x7c\x2e\xce\xcf\xa6\xd3\xc3\x85\xc2\x34\x1f\x59\x96\xa0\xed\x41\x10\xbb\x3a\x93\x95\xc1\x6c\xe0\xec\xe8\x1d\x88\xf8\x03\xa6\x4b\x06\xde\x64\xea\x22\xd8\x22\x11\x58\x5b\x4b\x9e\xcb\x21\x41\x5b\xd1\x02\x47\xec\x2e\x78\xd8\xbf\xac\xab\xc7\x13\x80\x67\x17\x4c\xb6\xc3\xb0\xa1\x74\x1e\x48\x84\xb3\x94\x7d\xb7\x26\x24\x83\xf9\x0a\x2b\x0c\x42\x9e\x41\x4f\x9c\xe1\xde\x50\x8b\x3d\x39\xaa\xf7\x88\x1d\x6c\x2b\x55\xd6\x20\x41\x1d\x35\x41\x55\x5f\xf5\xb0\xd5\x91\xee\xff\x73\xdb\xa2\x0c\xa2\xaf\x46\xc3\x51\x8c\x17\xbf\xb6\xfd\x12\x00\x03\xf4\x30\x82\x03\xf0\x30\x82\x02\xd8\xa0\x03\x02\x01\x02\x02\x03\x02\x3a\x83\x30\x0d\x06\x09\x2a\x86\x48\x86\xf7\x0d\x01\x01\x0b\x05\x00\x30\x42\x31\x0b\x30\x09\x06\x03\x55\x04\x06\x13\x02\x55\x53\x31\x16\x30\x14\x06\x03\x55\x04\x0a\x13\x0d\x47\x65\x6f\x54\x72\x75\x73\x74\x20\x49\x6e\x63\x2e\x31\x1b\x30\x19\x06\x03\x55\x04\x03\x13\x12\x47\x65\x6f\x54\x72\x75\x73\x74\x20\x47\x6c\x6f\x62\x61\x6c\x20\x43\x41\x30\x1e\x17\x0d\x31\x33\x30\x34\x30\x35\x31\x35\x31\x35\x35\x36\x5a\x17\x0d\x31\x36\x31\x32\x33\x31\x32\x33\x35\x39\x35\x39\x5a\x30\x49\x31\x0b\x30\x09\x06\x03\x55\x04\x06\x13\x02\x55\x53\x31\x13\x30\x11\x06\x03\x55\x04\x0a\x13\x0a\x47\x6f\x6f\x67\x6c\x65\x20\x49\x6e\x63\x31\x25\x30\x23\x06\x03\x55\x04\x03\x13\x1c\x47\x6f\x6f\x67\x6c\x65\x20\x49\x6e\x74\x65\x72\x6e\x65\x74\x20\x41\x75\x74\x68\x6f\x72\x69\x74\x79\x20\x47\x32\x30\x82\x01\x22\x30\x0d\x06\x09\x2a\x86\x48\x86\xf7\x0d\x01\x01\x01\x05\x00\x03\x82\x01\x0f\x00\x30\x82\x01\x0a\x02\x82\x01\x01\x00\x9c\x2a\x04\x77\x5c\xd8\x50\x91\x3a\x06\xa3\x82\xe0\xd8\x50\x48\xbc\x89\x3f\xf1\x19\x70\x1a\x88\x46\x7e\xe0\x8f\xc5\xf1\x89\xce\x21\xee\x5a\xfe\x61\x0d\xb7\x32\x44\x89\xa0\x74\x0b\x53\x4f\x55\xa4\xce\x82\x62\x95\xee\xeb\x59\x5f\xc6\xe1\x05\x80\x12\xc4\x5e\x94\x3f\xbc\x5b\x48\x38\xf4\x53\xf7\x24\xe6\xfb\x91\xe9\x15\xc4\xcf\xf4\x53\x0d\xf4\x4a\xfc\x9f\x54\xde\x7d\xbe\xa0\x6b\x6f\x87\xc0\xd0\x50\x1f\x28\x30\x03\x40\xda\x08\x73\x51\x6c\x7f\xff\x3a\x3c\xa7\x37\x06\x8e\xbd\x4b\x11\x04\xeb\x7d\x24\xde\xe6\xf9\xfc\x31\x71\xfb\x94\xd5\x60\xf3\x2e\x4a\xaf\x42\xd2\xcb\xea\xc4\x6a\x1a\xb2\xcc\x53\xdd\x15\x4b\x8b\x1f\xc8\x19\x61\x1f\xcd\x9d\xa8\x3e\x63\x2b\x84\x35\x69\x65\x84\xc8\x19\xc5\x46\x22\xf8\x53\x95\xbe\xe3\x80\x4a\x10\xc6\x2a\xec\xba\x97\x20\x11\xc7\x39\x99\x10\x04\xa0\xf0\x61\x7a\x95\x25\x8c\x4e\x52\x75\xe2\xb6\xed\x08\xca\x14\xfc\xce\x22\x6a\xb3\x4e\xcf\x46\x03\x97\x97\x03\x7e\xc0\xb1\xde\x7b\xaf\x45\x33\xcf\xba\x3e\x71\xb7\xde\xf4\x25\x25\xc2\x0d\x35\x89\x9d\x9d\xfb\x0e\x11\x79\x89\x1e\x37\xc5\xaf\x8e\x72\x69\x02\x03\x01\x00\x01\xa3\x81\xe7\x30\x81\xe4\x30\x1f\x06\x03\x55\x1d\x23\x04\x18\x30\x16\x80\x14\xc0\x7a\x98\x68\x8d\x89\xfb\xab\x05\x64\x0c\x11\x7d\xaa\x7d\x65\xb8\xca\xcc\x4e\x30\x1d\x06\x03\x55\x1d\x0e\x04\x16\x04\x14\x4a\xdd\x06\x16\x1b\xbc\xf6\x68\xb5\x76\xf5\x81\xb6\xbb\x62\x1a\xba\x5a\x81\x2f\x30\x0e\x06\x03\x55\x1d\x0f\x01\x01\xff\x04\x04\x03\x02\x01\x06\x30\x2e\x06\x08\x2b\x06\x01\x05\x05\x07\x01\x01\x04\x22\x30\x20\x30\x1e\x06\x08\x2b\x06\x01\x05\x05\x07\x30\x01\x86\x12\x68\x74\x74\x70\x3a\x2f\x2f\x67\x2e\x73\x79\x6d\x63\x64\x2e\x63\x6f\x6d\x30\x12\x06\x03\x55\x1d\x13\x01\x01\xff\x04\x08\x30\x06\x01\x01\xff\x02\x01\x00\x30\x35\x06\x03\x55\x1d\x1f\x04\x2e\x30\x2c\x30\x2a\xa0\x28\xa0\x26\x86\x24\x68\x74\x74\x70\x3a\x2f\x2f\x67\x2e\x73\x79\x6d\x63\x62\x2e\x63\x6f\x6d\x2f\x63\x72\x6c\x73\x2f\x67\x74\x67\x6c\x6f\x62\x61\x6c\x2e\x63\x72\x6c\x30\x17\x06\x03\x55\x1d\x20\x04\x10\x30\x0e\x30\x0c\x06\x0a\x2b\x06\x01\x04\x01\xd6\x79\x02\x05\x01\x30\x0d\x06\x09\x2a\x86\x48\x86\xf7\x0d\x01\x01\x0b\x05\x00\x03\x82\x01\x01\x00\xaa\xfa\xa9\x20\xcd\x6a\x67\x83\xed\x5e\xd4\x7e\xde\x1d\xc4\x7f\xe0\x25\x06\x00\xc5\x24\xfb\xa9\xc8\x2d\x6d\x7e\xde\x9d\x82\x65\x2c\x81\x63\x34\x66\x3e\xe9\x52\xc2\x08\xb4\xcb\x2f\xf7\x5f\x99\x3a\x6a\x9c\x50\x7a\x85\x05\x8c\x7d\xd1\x2a\x48\x84\xd3\x09\x6c\x7c\xc2\xcd\x35\x9f\xf3\x82\xee\x52\xde\x68\x5f\xe4\x00\x8a\x17\x20\x96\xf7\x29\x8d\x9a\x4d\xcb\xa8\xde\x86\xc8\x0d\x6f\x56\x87\x03\x7d\x03\x3f\xdc\xfa\x79\x7d\x21\x19\xf9\xc8\x3a\x2f\x51\x76\x8c\xc7\x41\x92\x71\x8f\x25\xce\x37\xf8\x4a\x4c\x00\x23\xef\xc4\x35\x10\xae\xe0\x23\x80\x73\x7c\x4d\x34\x2e\xc8\x6e\x90\xd6\x10\x1e\x99\x84\x73\x1a\x70\xf2\xed\x55\x0e\xee\x17\x06\xea\x67\xee\x32\xeb\x2c\xdd\x67\x07\x3f\xf6\x8b\xc2\x70\xde\x5b\x00\xe6\xbb\x1b\xd3\x36\x1a\x22\x6c\x6c\xb0\x35\x42\x6c\x90\x09\x3d\x93\xe9\x64\x09\x22\x0e\x85\x06\x9f\xc2\x73\x21\xd3\xe6\x5f\x80\xe4\x8d\x85\x22\x3a\x73\x03\xb1\x60\x8e\xae\x68\xe2\xf4\x3e\x97\xe7\x60\x12\x09\x68\x36\xde\x3a\xd6\xe2\x43\x95\x5b\x37\x81\x92\x81\x1f\xbb\x8d\xd7\xad\x52\x64\x16\x57\x96\xd9\x5e\x34\x7e\xc8\x35\xd8\x00\x03\x81\x30\x82\x03\x7d\x30\x82\x02\xe6\xa0\x03\x02\x01\x02\x02\x03\x12\xbb\xe6\x30\x0d\x06\x09\x2a\x86\x48\x86\xf7\x0d\x01\x01\x05\x05\x00\x30\x4e\x31\x0b\x30\x09\x06\x03\x55\x04\x06\x13\x02\x55\x53\x31\x10\x30\x0e\x06\x03\x55\x04\x0a\x13\x07\x45\x71\x75\x69\x66\x61\x78\x31\x2d\x30\x2b\x06\x03\x55\x04\x0b\x13\x24\x45\x71\x75\x69\x66\x61\x78\x20\x53\x65\x63\x75\x72\x65\x20\x43\x65\x72\x74\x69\x66\x69\x63\x61\x74\x65\x20\x41\x75\x74\x68\x6f\x72\x69\x74\x79\x30\x1e\x17\x0d\x30\x32\x30\x35\x32\x31\x30\x34\x30\x30\x30\x30\x5a\x17\x0d\x31\x38\x30\x38\x32\x31\x30\x34\x30\x30\x30\x30\x5a\x30\x42\x31\x0b\x30\x09\x06\x03\x55\x04\x06\x13\x02\x55\x53\x31\x16\x30\x14\x06\x03\x55\x04\x0a\x13\x0d\x47\x65\x6f\x54\x72\x75\x73\x74\x20\x49\x6e\x63\x2e\x31\x1b\x30\x19\x06\x03\x55\x04\x03\x13\x12\x47\x65\x6f\x54\x72\x75\x73\x74\x20\x47\x6c\x6f\x62\x61\x6c\x20\x43\x41\x30\x82\x01\x22\x30\x0d\x06\x09\x2a\x86\x48\x86\xf7\x0d\x01\x01\x01\x05\x00\x03\x82\x01\x0f\x00\x30\x82\x01\x0a\x02\x82\x01\x01\x00\xda\xcc\x18\x63\x30\xfd\xf4\x17\x23\x1a\x56\x7e\x5b\xdf\x3c\x6c\x38\xe4\x71\xb7\x78\x91\xd4\xbc\xa1\xd8\x4c\xf8\xa8\x43\xb6\x03\xe9\x4d\x21\x07\x08\x88\xda\x58\x2f\x66\x39\x29\xbd\x05\x78\x8b\x9d\x38\xe8\x05\xb7\x6a\x7e\x71\xa4\xe6\xc4\x60\xa6\xb0\xef\x80\xe4\x89\x28\x0f\x9e\x25\xd6\xed\x83\xf3\xad\xa6\x91\xc7\x98\xc9\x42\x18\x35\x14\x9d\xad\x98\x46\x92\x2e\x4f\xca\xf1\x87\x43\xc1\x16\x95\x57\x2d\x50\xef\x89\x2d\x80\x7a\x57\xad\xf2\xee\x5f\x6b\xd2\x00\x8d\xb9\x14\xf8\x14\x15\x35\xd9\xc0\x46\xa3\x7b\x72\xc8\x91\xbf\xc9\x55\x2b\xcd\xd0\x97\x3e\x9c\x26\x64\xcc\xdf\xce\x83\x19\x71\xca\x4e\xe6\xd4\xd5\x7b\xa9\x19\xcd\x55\xde\xc8\xec\xd2\x5e\x38\x53\xe5\x5c\x4f\x8c\x2d\xfe\x50\x23\x36\xfc\x66\xe6\xcb\x8e\xa4\x39\x19\x00\xb7\x95\x02\x39\x91\x0b\x0e\xfe\x38\x2e\xd1\x1d\x05\x9a\xf6\x4d\x3e\x6f\x0f\x07\x1d\xaf\x2c\x1e\x8f\x60\x39\xe2\xfa\x36\x53\x13\x39\xd4\x5e\x26\x2b\xdb\x3d\xa8\x14\xbd\x32\xeb\x18\x03\x28\x52\x04\x71\xe5\xab\x33\x3d\xe1\x38\xbb\x07\x36\x84\x62\x9c\x79\xea\x16\x30\xf4\x5f\xc0\x2b\xe8\x71\x6b\xe4\xf9\x02\x03\x01\x00\x01\xa3\x81\xf0\x30\x81\xed\x30\x1f\x06\x03\x55\x1d\x23\x04\x18\x30\x16\x80\x14\x48\xe6\x68\xf9\x2b\xd2\xb2\x95\xd7\x47\xd8\x23\x20\x10\x4f\x33\x98\x90\x9f\xd4\x30\x1d\x06\x03\x55\x1d\x0e\x04\x16\x04\x14\xc0\x7a\x98\x68\x8d\x89\xfb\xab\x05\x64\x0c\x11\x7d\xaa\x7d\x65\xb8\xca\xcc\x4e\x30\x0f\x06\x03\x55\x1d\x13\x01\x01\xff\x04\x05\x30\x03\x01\x01\xff\x30\x0e\x06\x03\x55\x1d\x0f\x01\x01\xff\x04\x04\x03\x02\x01\x06\x30\x3a\x06\x03\x55\x1d\x1f\x04\x33\x30\x31\x30\x2f\xa0\x2d\xa0\x2b\x86\x29\x68\x74\x74\x70\x3a\x2f\x2f\x63\x72\x6c\x2e\x67\x65\x6f\x74\x72\x75\x73\x74\x2e\x63\x6f\x6d\x2f\x63\x72\x6c\x73\x2f\x73\x65\x63\x75\x72\x65\x63\x61\x2e\x63\x72\x6c\x30\x4e\x06\x03\x55\x1d\x20\x04\x47\x30\x45\x30\x43\x06\x04\x55\x1d\x20\x00\x30\x3b\x30\x39\x06\x08\x2b\x06\x01\x05\x05\x07\x02\x01\x16\x2d\x68\x74\x74\x70\x73\x3a\x2f\x2f\x77\x77\x77\x2e\x67\x65\x6f\x74\x72\x75\x73\x74\x2e\x63\x6f\x6d\x2f\x72\x65\x73\x6f\x75\x72\x63\x65\x73\x2f\x72\x65\x70\x6f\x73\x69\x74\x6f\x72\x79\x30\x0d\x06\x09\x2a\x86\x48\x86\xf7\x0d\x01\x01\x05\x05\x00\x03\x81\x81\x00\x76\xe1\x12\x6e\x4e\x4b\x16\x12\x86\x30\x06\xb2\x81\x08\xcf\xf0\x08\xc7\xc7\x71\x7e\x66\xee\xc2\xed\xd4\x3b\x1f\xff\xf0\xf0\xc8\x4e\xd6\x43\x38\xb0\xb9\x30\x7d\x18\xd0\x55\x83\xa2\x6a\xcb\x36\x11\x9c\xe8\x48\x66\xa3\x6d\x7f\xb8\x13\xd4\x47\xfe\x8b\x5a\x5c\x73\xfc\xae\xd9\x1b\x32\x19\x38\xab\x97\x34\x14\xaa\x96\xd2\xeb\xa3\x1c\x14\x08\x49\xb6\xbb\xe5\x91\xef\x83\x36\xeb\x1d\x56\x6f\xca\xda\xbc\x73\x63\x90\xe4\x7f\x7b\x3e\x22\xcb\x3d\x07\xed\x5f\x38\x74\x9c\xe3\x03\x50\x4e\xa1\xaf\x98\xee\x61\xf2\x84\x3f\x12\x16\x03\x03\x00\x93\x0c\x00\x00\x8f\x03\x00\x17\x41\x04\xd3\x0c\xa0\x09\xc0\x15\x9b\x47\xdf\x87\xc4\x5c\xfa\xed\x83\xee\x7b\xa7\xd7\x74\xb2\x55\xc6\x96\x50\x4c\xe9\xe4\x4c\xb0\x4c\x13\x5b\xa5\xd2\x0e\x5e\x0b\x0b\xb7\xbe\x61\x26\x62\x51\x97\x49\xe4\x91\x29\xaf\xa9\x15\xda\x73\x08\xa6\x9d\x0f\x5a\x30\x2b\x78\xd0\x04\x03\x00\x46\x30\x44\x02\x20\x76\x48\xde\x11\x21\xb7\xd0\xde\x05\x8e\xaf\x27\xc1\x0e\xbf\x2a\x0a\x1f\x3d\xb7\xad\x9b\xed\x58\x03\xda\x07\x5c\x8d\x00\x2b\x47\x02\x20\x6a\x06\x4e\x56\x66\x57\x27\x79\xf9\x21\x21\x31\x66\x32\x52\xed\x4b\x5f\xce\x65\xe9\xaa\xb1\xc6\x11\xbd\x49\x39\xe1\x6e\x2a\x43\x16\x03\x03\x00\x04\x0e\x00\x00\x00");

    while r.any_left() {
      let mut m = Message::read(&mut r).unwrap();

      let mut out: Vec<u8> = vec![];
      m.encode(&mut out);
      assert!(out.len() > 0);

      m.decode_payload();
    }
  }
}

use msgs::enums::{AlertLevel, AlertDescription};
use msgs::codec::{Codec, Reader};

#[derive(Debug)]
pub struct AlertMessagePayload {
  pub level: AlertLevel,
  pub description: AlertDescription
}

impl Codec for AlertMessagePayload {
  fn encode(&self, bytes: &mut Vec<u8>) {
    self.level.encode(bytes);
    self.description.encode(bytes);
  }

  fn read(r: &mut Reader) -> Option<AlertMessagePayload> {
    let level = try_ret!(AlertLevel::read(r));
    let desc = try_ret!(AlertDescription::read(r));

    Some(AlertMessagePayload { level: level, description: desc })
  }
}

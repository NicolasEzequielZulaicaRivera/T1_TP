use std::io::Read;

use self::packet_reader::{ErrorKind, PacketError};
mod packet_reader;
#[path = "../utf8.rs"]
mod utf8;
use crate::connect::utf8::Field;

/*
const MAX_PAYLOAD_FIELD_LEN: usize = 65535;
const CONNECT_FIXED_HEADER_TYPE: u8 = 0x01;
const CONNECT_FIXED_HEADER_FLAGS: u8 = 0x00;
const SHIFT: u8 = 4;
const PROTOCOL_LEVEL_3_1_1: u8 = 0x04;
*/

const RESERVED: u8 = 0x1;
const CLEAN_SESSION: u8 = 0x2;
const WILL_FLAG: u8 = 0x4;
const WILL_QOS_0: u8 = 0x8;
const WILL_QOS_1: u8 = 0x10;
const WILL_RETAIN: u8 = 0x20;
const PASSWORD_FLAG: u8 = 0x40;
const USERNAME_FLAG: u8 = 0x80;

#[derive(Debug)]
pub enum QoSLevel {
    QoSLevel0,
    QoSLevel1,
}

#[derive(Debug)]
pub struct LastWill {
    pub retain: bool,
    pub qos: QoSLevel,
    pub topic: String,
    pub message: String,
}

#[derive(Debug)]
pub struct Connect {
    client_id: String,
    clean_session: bool,
    user_name: Option<String>,
    password: Option<String>,
    last_will: Option<LastWill>,
    keep_alive: u16,
}

const PROTOCOL_LEVEL: u8 = 4;

impl Connect {
    fn verify_protocol(bytes: &mut impl Read) -> Result<(), PacketError> {
        match Field::new_from_stream(bytes) {
            Some(mensaje) if mensaje.value != "MQTT" => Err(PacketError::new_kind(
                "Invalid protocol",
                ErrorKind::InvalidProtocol,
            )),
            None => Err(PacketError::new()),
            Some(_mensaje) => Ok(()),
        }
    }

    fn verify_protocol_level(bytes: &mut impl Read) -> Result<(), PacketError> {
        let mut buf = [0; 1];
        bytes.read_exact(&mut buf)?;
        if buf[0] != PROTOCOL_LEVEL {
            return Err(PacketError::new_kind(
                "Invalid protocol level",
                ErrorKind::InvalidProtocolLevel,
            ));
        }
        Ok(())
    }

    fn get_will(buf: [u8; 1]) -> Result<Option<LastWill>, PacketError> {
        if buf[0] & WILL_FLAG != 0 {
            return Ok(Some(LastWill {
                retain: buf[0] & WILL_RETAIN != 0,
                qos: if buf[0] & WILL_QOS_0 == 0 {
                    QoSLevel::QoSLevel0
                } else {
                    QoSLevel::QoSLevel1
                },
                topic: String::new(),
                message: String::new(),
            }));
        }

        if buf[0] & WILL_QOS_0 != 0 || buf[0] & WILL_QOS_1 != 0 || buf[0] & WILL_RETAIN != 0 {
            return Err(PacketError::new_kind(
                "Invalid flags",
                ErrorKind::InvalidFlags,
            ));
        }
        Ok(None)
    }

    fn get_flags(bytes: &mut impl Read) -> Result<Connect, PacketError> {
        let mut buf = [0; 1];
        bytes.read_exact(&mut buf)?;

        if buf[0] & RESERVED != 0 {
            return Err(PacketError::new_kind(
                "Reserved bits are not zero",
                ErrorKind::InvalidFlags,
            ));
        }

        Ok(Connect {
            client_id: String::new(),
            clean_session: buf[0] & CLEAN_SESSION != 0,
            user_name: if buf[0] & USERNAME_FLAG != 0 {
                Some(String::new())
            } else {
                None
            },
            password: if buf[0] & PASSWORD_FLAG != 0 {
                Some(String::new())
            } else {
                None
            },
            last_will: Connect::get_will(buf)?,
            keep_alive: 0,
        })
    }

    fn get_keepalive(&mut self, bytes: &mut impl Read) -> Result<(), PacketError> {
        let mut buf = [0; 2];
        bytes.read_exact(&mut buf)?;
        self.keep_alive = u16::from_be_bytes(buf);
        Ok(())
    }

    fn get_clientid(&mut self, bytes: &mut impl Read) -> Result<(), PacketError> {
        let string = Field::new_from_stream(bytes).ok_or_else(PacketError::new)?;
        self.client_id = string.value;
        Ok(())
    }

    fn get_will_data(&mut self, bytes: &mut impl Read) -> Result<(), PacketError> {
        if let Some(lw) = &mut self.last_will {
            let topic = Field::new_from_stream(bytes).ok_or_else(PacketError::new)?;
            let message = Field::new_from_stream(bytes).ok_or_else(PacketError::new)?;
            lw.topic = topic.value;
            lw.message = message.value;
        }
        Ok(())
    }

    fn get_auth(&mut self, bytes: &mut impl Read) -> Result<(), PacketError> {
        if let Some(user_name) = &mut self.user_name {
            let user = Field::new_from_stream(bytes).ok_or_else(PacketError::new)?;
            *user_name = user.value;
        }
        if let Some(pw) = &mut self.password {
            let password = Field::new_from_stream(bytes).ok_or_else(PacketError::new)?;
            *pw = password.value;
        }
        Ok(())
    }

    pub fn new(fixed_header: [u8; 2], stream: &mut impl Read) -> Result<Connect, PacketError> {
        let mut bytes = packet_reader::read_packet_bytes(fixed_header, stream)?;

        Connect::verify_protocol(&mut bytes)?;
        Connect::verify_protocol_level(&mut bytes)?;
        let mut ret = Connect::get_flags(&mut bytes)?;
        ret.get_keepalive(&mut bytes)?;
        ret.get_clientid(&mut bytes)?;
        ret.get_will_data(&mut bytes)?;
        ret.get_auth(&mut bytes)?;
        
        let mut buf = [0u8; 1];
        match bytes.read(&mut buf) {
            Ok(1) | Err(_) => {
                // Sobraron bytes, no debería
                return Err(PacketError::new());
            }
            Ok(_) => (), // No sobro, perfecto
        }
        
        Ok(ret)
    }

    pub fn response(&self) {
        //TODO
    }

    /// Get a reference to the connect's client id.
    pub fn client_id(&self) -> &str {
        self.client_id.as_str()
    }

    /// Get a reference to the connect's clean session.
    pub fn clean_session(&self) -> &bool {
        &self.clean_session
    }

    /// Get a reference to the connect's user name.
    pub fn user_name(&self) -> Option<&String> {
        self.user_name.as_ref()
    }

    /// Get a reference to the connect's password.
    pub fn password(&self) -> Option<&String> {
        self.password.as_ref()
    }

    /// Get a reference to the connect's last will.
    pub fn last_will(&self) -> Option<&LastWill> {
        self.last_will.as_ref()
    }

    /// Get a reference to the connect's keep alive.
    pub fn keep_alive(&self) -> &u16 {
        &self.keep_alive
    }
}

#[cfg(test)]
mod tests {

    use super::utf8::Field;
    use super::ErrorKind;
    use crate::{
        connect::{
            Connect, CLEAN_SESSION, PASSWORD_FLAG, RESERVED, USERNAME_FLAG, WILL_FLAG, WILL_QOS_0,
            WILL_QOS_1, WILL_RETAIN,
        }
    };
    use std::io::Cursor;

    const HEADER_1: u8 = 0b00010000;

    #[test]
    fn test_keep_alive() {
        let mut v = Field::new_from_string("MQTT").encode();
        v.push(4u8); // Nivel
        v.push(0u8); //Flags
        v.append(&mut vec![16u8, 60u8]); //Keep alive
        v.append(&mut Field::new_from_string("id").encode());

        let headers = [HEADER_1, v.len() as u8];
        let mut bytes = Cursor::new(v);

        assert_eq!(
            *Connect::new(headers, &mut bytes).unwrap().keep_alive(),
            ((16 << 8) + 60) as u16
        );
    }

    #[test]
    fn test_invalid_protocol() {
        let mut v = Field::new_from_string("Not MQTT").encode();
        v.push(4u8); // Nivel
        v.push(0u8); //Flags
        v.append(&mut vec![0u8, 60u8]); //Keep alive
        v.append(&mut Field::new_from_string("id").encode());

        let headers = [HEADER_1, v.len() as u8];
        let mut bytes = Cursor::new(v);

        assert_eq!(
            Connect::new(headers, &mut bytes).unwrap_err().kind(),
            ErrorKind::InvalidProtocol
        );
    }

    #[test]
    fn test_invalid_protocol_level() {
        let mut v = Field::new_from_string("MQTT").encode();
        v.push(7u8); // Nivel
        v.push(0u8); //Flags
        v.append(&mut vec![0u8, 60u8]); //Keep alive
        v.append(&mut Field::new_from_string("id").encode());

        let headers = [HEADER_1, v.len() as u8];
        let mut bytes = Cursor::new(v);

        assert_eq!(
            Connect::new(headers, &mut bytes).unwrap_err().kind(),
            ErrorKind::InvalidProtocolLevel
        );
    }

    #[test]
    fn test_invalid_reserved_flag() {
        let mut v = Field::new_from_string("MQTT").encode();
        v.push(4u8); // Nivel
        v.push(RESERVED); //Flags
        v.append(&mut vec![0u8, 60u8]); //Keep alive
        v.append(&mut Field::new_from_string("id").encode());

        let headers = [HEADER_1, v.len() as u8];
        let mut bytes = Cursor::new(v);

        assert_eq!(
            Connect::new(headers, &mut bytes).unwrap_err().kind(),
            ErrorKind::InvalidFlags
        );
    }

    #[test]
    fn test_will_flag_0_topic_message_1() {
        let mut v = Field::new_from_string("MQTT").encode();
        v.push(4u8); // Nivel
        v.push(WILL_QOS_0 | WILL_QOS_1 | WILL_RETAIN); //Flags
        v.append(&mut Field::new_from_string("id").encode());
        v.append(&mut vec![0u8, 60u8]); //Keep alive

        let headers = [HEADER_1, v.len() as u8];
        let mut bytes = Cursor::new(v);

        assert_eq!(
            Connect::new(headers, &mut bytes).unwrap_err().kind(),
            ErrorKind::InvalidFlags
        );
    }

    #[test]
    fn test_username_missing_but_needed() {
        let mut v = Field::new_from_string("MQTT").encode();
        v.push(4u8); // Nivel
        v.push(USERNAME_FLAG); //Flags
        v.append(&mut vec![0u8, 60u8]); //Keep alive
        v.append(&mut Field::new_from_string("id").encode());

        let headers = [HEADER_1, v.len() as u8];
        let mut bytes = Cursor::new(v);

        assert!(Connect::new(headers, &mut bytes).is_err());
    }

    #[test]
    fn test_username_present_but_shouldnt() {
        let mut v = Field::new_from_string("MQTT").encode();
        v.push(4u8); // Nivel
        v.push(0); //Flags
        v.append(&mut vec![0u8, 60u8]); //Keep alive
        v.append(&mut Field::new_from_string("id").encode());
        v.append(&mut Field::new_from_string("unNombre").encode());

        let headers = [HEADER_1, v.len() as u8];
        let mut bytes = Cursor::new(v);

        assert!(Connect::new(headers, &mut bytes).is_err());
    }

    #[test]
    fn test_connect_clean_session() {
        let mut v = Field::new_from_string("MQTT").encode();
        v.push(4u8); // Nivel
        v.push(CLEAN_SESSION); //Flags
        v.append(&mut vec![0u8, 60u8]); //Keep alive
        v.append(&mut Field::new_from_string("id").encode());

        let headers = [HEADER_1, v.len() as u8];
        let mut bytes = Cursor::new(v);

        assert!(Connect::new(headers, &mut bytes).unwrap().clean_session());
    }

    #[test]
    fn test_will_flag() {
        let mut v = Field::new_from_string("MQTT").encode();
        v.push(4u8); // Nivel
        v.push(WILL_FLAG | WILL_RETAIN); //Flags
        v.append(&mut vec![0u8, 60u8]); //Keep alive
        v.append(&mut Field::new_from_string("id").encode());
        v.append(&mut Field::new_from_string("soyUnTopic").encode());
        v.append(&mut Field::new_from_string("soyUnMensaje").encode());

        let headers = [HEADER_1, v.len() as u8];
        let mut bytes = Cursor::new(v);
        let packet = Connect::new(headers, &mut bytes).unwrap();
        let will = packet.last_will().unwrap();

        assert!(will.retain);
        assert_eq!(will.topic, "soyUnTopic");
        assert_eq!(will.message, "soyUnMensaje");
    }

    #[test]
    fn test_will_flag_username_password() {
        let mut v = Field::new_from_string("MQTT").encode();
        v.push(4u8); // Nivel
        v.push(WILL_FLAG | WILL_RETAIN | USERNAME_FLAG | PASSWORD_FLAG); //Flags
        v.append(&mut vec![0u8, 60u8]); //Keep alive
        v.append(&mut Field::new_from_string("id").encode());
        v.append(&mut Field::new_from_string("soyUnTopic").encode());
        v.append(&mut Field::new_from_string("soyUnMensaje").encode());
        v.append(&mut Field::new_from_string("siAlguienLeeEstoFelicitaciones").encode());
        v.append(&mut Field::new_from_string("contraseñaSuperSecreta").encode());

        let headers = [HEADER_1, v.len() as u8];
        let mut bytes = Cursor::new(v);
        let packet = Connect::new(headers, &mut bytes).unwrap();
        let will = packet.last_will().unwrap();

        assert!(will.retain);
        assert_eq!(will.topic, "soyUnTopic");
        assert_eq!(will.message, "soyUnMensaje");
        assert_eq!(
            packet.user_name().unwrap(),
            "siAlguienLeeEstoFelicitaciones"
        );
        assert_eq!(packet.password().unwrap(), "contraseñaSuperSecreta");
    }
}

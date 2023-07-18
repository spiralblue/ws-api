use chrono::prelude::*;
use cobs::{decode_vec, encode_vec};

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(u8)]
pub enum CommandType {
    Time = 0,
    StartupCommand = 1,
    Initialised = 2,
    PowerDown = 3,
    TimeAcknowledge = 4,
    StartupCommandAcknowledge = 5,
    InitialisedAcknowledge = 6,
    PowerDownAcknowledge = 7,
}

impl From<u8> for CommandType {
    fn from(byte: u8) -> CommandType {
        match byte {
            0 => CommandType::Time,
            1 => CommandType::StartupCommand,
            2 => CommandType::Initialised,
            3 => CommandType::PowerDown,
            4 => CommandType::TimeAcknowledge,
            5 => CommandType::StartupCommandAcknowledge,
            6 => CommandType::InitialisedAcknowledge,
            7 => CommandType::PowerDownAcknowledge,
            _ => panic!("Invalid command type"),
        }
    }
}

pub struct Command {
    pub command_type: CommandType,
    pub data: Vec<u8>,
}

pub fn datetime_to_bytes(time: DateTime<Utc>) -> Vec<u8> {
    let time = time.timestamp_millis();
    time.to_be_bytes().to_vec()
}

pub fn bytes_to_datetime(bytes: &[u8]) -> DateTime<Utc> {
    let mut time_bytes = [0u8; 8];
    time_bytes.copy_from_slice(&bytes[..8]);
    let time = i64::from_be_bytes(time_bytes);
    Utc.timestamp_millis_opt(time).unwrap()
}

impl Command {
    pub fn new(command_type: CommandType, data: Vec<u8>) -> Command {
        Command {
            command_type,
            data,
        }
    }

    pub fn time(time: DateTime<Utc>) -> Command {
        Command::new(CommandType::Time, datetime_to_bytes(time))
    }

    pub fn startup_command(command: Vec<u8>) -> Command {
        Command::new(CommandType::StartupCommand, command)
    }

    pub fn simple_command(command_type: CommandType) -> Command {
        Command::new(command_type, Vec::new())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.command_type as u8);
        bytes.extend(self.data.iter());

        // COBS encode ( decode in python with https://github.com/cmcqueen/cobs-python/ )
        let mut encoded = encode_vec(&bytes);
        encoded.push(0);  // Add a null byte to the end to indicate end of command
        encoded
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Command {
        let null_index = bytes.iter().position(|&x| x == 0).unwrap();
        let decoded = decode_vec(&bytes[0..null_index].to_vec()).unwrap();
        let command_type = decoded[0];
        let data = decoded[1..].to_vec();
        Command::new(command_type.into(), data)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_datetime_bytes_encoding() {
        for offset in [-100, 0, 100].iter() {
            let time = Utc::now() + chrono::Duration::milliseconds(*offset);
            let bytes = datetime_to_bytes(time);
            let decoded_time = bytes_to_datetime(&bytes);
            assert_eq!(decoded_time.timestamp_millis(), time.timestamp_millis());
        }
    }

    #[test]
    fn test_command_encoding() {
        for command_type in [CommandType::Time, CommandType::StartupCommand].iter() {
            for data in [vec![1, 2, 3], vec![4, 5, 6]].iter() {
                let command = Command::new(*command_type, data.clone());
                let bytes = command.to_bytes();
                let decoded = Command::from_bytes(bytes);
                assert_eq!(decoded.command_type, *command_type);
                assert_eq!(decoded.data, *data);
            }
        }
    }

    #[test]
    fn test_time() {
        for offset in [-100, 0, 100].iter() {
            let time = Utc::now() + chrono::Duration::milliseconds(*offset);
            let command = Command::time(time);
            let bytes = command.to_bytes();
            let decoded = Command::from_bytes(bytes);
            assert_eq!(decoded.command_type, CommandType::Time);
            let decoded_time = bytes_to_datetime(&decoded.data);
            assert_eq!(decoded_time.timestamp_millis(), time.timestamp_millis());
        }
    }

    #[test]
    fn test_startup_command() {
        for startup_command in ["patch01.json", "orbit05.json", "asdfGHJK.json"].iter() {
            let command = Command::startup_command(startup_command.as_bytes().to_vec());
            let bytes = command.to_bytes();
            let decoded = Command::from_bytes(bytes);
            assert_eq!(decoded.command_type, CommandType::StartupCommand);
            assert_eq!(decoded.data, startup_command.as_bytes());
        }
    }

    #[test]
    fn test_simple_command() {
        for command_type in [CommandType::Initialised, CommandType::PowerDown, CommandType::TimeAcknowledge, CommandType::StartupCommandAcknowledge, CommandType::InitialisedAcknowledge, CommandType::StartupCommandAcknowledge].iter() {
            let command = Command::simple_command(*command_type);
            let bytes = command.to_bytes();
            let decoded = Command::from_bytes(bytes);
            assert_eq!(decoded.command_type, *command_type);
            assert_eq!(decoded.data, Vec::new());
        }
    }
}

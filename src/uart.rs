use std::time::{Duration, Instant};
use serial::*;
// use uart_rs::{Connection, UartResult};
use crate::{Command, CommandType, Ftp};
use std::io::{Read, Write};
use std::fs::File;
use serial::{SerialPort, SerialPortSettings};
use sha2::{Digest, Sha256};

const UART_RECEIVE_TIMEOUT: Duration = Duration::from_secs(1);

pub struct UartConnection {
    // port: Box<dyn SerialPort>,
    path: String,
    settings: PortSettings,
    timeout: Duration,
}

impl UartConnection {
    /// Create a new UartConnection
    ///
    /// # Arguments
    ///
    /// * `uart_path` - The path to the UART device
    /// * `uart_setting` - The settings of the UART device
    /// * `uart_timeout` - The timeout of the UART device
    ///
    /// # Returns
    ///
    /// * A new UartConnection
    ///
    pub fn new(
        uart_path: String,
        uart_setting: PortSettings,
        uart_timeout: Duration,
    ) -> std::io::Result<Self> {
        // let mut port = serial::open(&uart_path)?;
        // port.configure(&uart_setting)?;
        // port.set_timeout(uart_timeout)?;
        Ok(Self {
            path: uart_path,
            settings: uart_setting,
            timeout: uart_timeout,
        })
    }

    /// Send a message to the UART device
    ///
    /// # Arguments
    ///
    /// * `command` - The command to send
    ///
    /// # Returns
    ///
    /// * A UartResult containing the result of the send
    ///
    pub fn send_message(&mut self, command: Command) -> std::io::Result<()> {
        let data = command.to_bytes();
        let mut port = serial::open(&self.path)?;
        port.configure(&self.settings)?;
        port.set_timeout(self.timeout)?;
        match port.write(&data) {
            Ok(_) => {
                println!("Sent: {:?}", data);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Receive a message from the UART device
    ///
    /// # Arguments
    ///
    /// * `timeout` - The timeout of the receive
    ///
    /// # Returns
    ///
    /// * An Option containing the received message
    ///
    pub fn receive_message(&mut self, timeout: Duration) -> std::io::Result<Option<Command>> {
        let mut port = serial::open(&self.path)?;
        port.configure(&self.settings)?;
        port.set_timeout(self.timeout)?;
        let start_time = Instant::now();
        let mut data = Vec::new();
        loop {
            if start_time.elapsed() > timeout {
                break;
            }
            let mut buffer = [0u8; 1];
            if let Ok(response) = port.read(&mut buffer) {
                let byte = buffer[0];
                data.push(byte);
                if byte == 0 {
                    break;
                }
            }
        }
        // println!("Received: {:?}", data);
        Ok(Command::from_bytes(data))
    }

    pub fn receive_init(&mut self, timeout: Duration) -> std::io::Result<Vec<u8>> {
        let mut port = serial::open(&self.path)?;
        port.configure(&self.settings)?;
        port.set_timeout(self.timeout)?;
        let start_time = Instant::now();
        let mut data = Vec::new();
        loop {
            if start_time.elapsed() > timeout {
                break;
            }
            let mut buffer = [0u8; 1];
            if let Ok(response) = port.read(&mut buffer) {
                let byte = buffer[0];
                data.push(byte);
            }
        }
        // println!("Received: {:?}", data);
        Ok(data)
    }
}

impl Read for UartConnection {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let mut port = serial::open(&self.path)?;
        port.configure(&self.settings)?;
        port.set_timeout(self.timeout)?;
        Ok(port.read(buffer)?)
    }
}

impl Write for UartConnection {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut port = serial::open(&self.path)?;
        port.configure(&self.settings)?;
        port.set_timeout(self.timeout)?;
        port.write(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut port = serial::open(&self.path)?;
        port.configure(&self.settings)?;
        port.set_timeout(self.timeout)?;
        Ok(port.flush()?)
        // Ok(())
    }
}

impl Ftp for UartConnection {
    fn ftp(&mut self) -> std::io::Result<()> {
        let mut buffer = [0; 1024];
        let mut file_name = String::new();

        // Receive file name
        loop {
            let bytes_read = self.read(&mut buffer)?;
            file_name.push_str(std::str::from_utf8(&buffer[..bytes_read]).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?);
            if bytes_read < buffer.len() {
                break;
            }
        }

        // Remove trailing null bytes and any directory path
        file_name = file_name.trim_end_matches(char::from(0)).rsplit('/').next().unwrap().to_string();

        // Send READY_RECEIVE_FILE message
        self.write_all(b"READY_RECEIVE_FILE")?;

        // Receive file data
        let mut file_data = Vec::new();
        loop {
            let bytes_read = self.read(&mut buffer)?;
            file_data.extend_from_slice(&buffer[..bytes_read]);
            if bytes_read < buffer.len() {
                break;
            }
        }

        // Send RECEIVED_FILE_DATA message
        self.write_all(b"RECEIVED_FILE_DATA")?;

        // Compute file hash
        let file_hash = Sha256::digest(&file_data);

        // Send SEND_FILE_HASH message
        self.write_all(b"SEND_FILE_HASH")?;

        // Receive file hash
        let mut hash_buffer = [0; 32];
        self.read_exact(&mut hash_buffer)?;

        // Check file hash
        if hash_buffer != file_hash.as_slice() {
            self.write_all(b"RECEIVE_FILE_ERROR_RETRY")?;
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "File hash does not match"));
        }

        // Send RECEIVE_FILE_SUCCESS message
        self.write_all(b"RECEIVE_FILE_SUCCESS")?;

        // Write file data to disk
        let mut file = File::create(&file_name)?;
        file.write_all(&file_data)?;

        Ok(())
    }
}
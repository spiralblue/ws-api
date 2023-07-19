use std::time::{Duration, Instant};
use serial::PortSettings;
use uart_rs::{Connection, UartResult};
use crate::{Command, CommandType};


const UART_RECEIVE_TIMEOUT: Duration = Duration::from_millis(1);


pub struct UartConnection {
    connection: Connection,
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
    ) -> Self {
        Self {
            connection: Connection::from_path(&uart_path, uart_setting, uart_timeout),
        }
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
    pub fn send_message(&mut self, command: Command) -> UartResult<()> {
        let data = command.to_bytes();
        return self.connection.write(&data);
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
    pub fn receive_message(&mut self, timeout: Duration) -> Option<Command> {
        let start_time = Instant::now();
        let mut data = Vec::new();
        loop {
            if start_time.elapsed() > timeout {
                break;
            }
            if let Ok(response) = self.connection.read(1, UART_RECEIVE_TIMEOUT) {
                let byte = response[0];
                data.push(byte[0]);
                if byte == 0 {
                    break;
                }
            }
        }
        return Command::from_bytes(data);
    }
}
use std::os::unix::net::UnixStream;
use std::io::{Write, Read, Result};
use std::time;
use std::path::PathBuf;
use std::env;
use std::fmt::Debug;
use models::Payload;

use models::Message;
use super::base::Connection;

pub struct UnixConnection {
    socket: UnixStream,
}

impl UnixConnection {
    fn ipc_path() -> PathBuf {
        let tmp = env::var("XDG_RUNTIME_DIR")
            .or_else(|_| env::var("TMPDIR"))
            .or_else(|_| {
                match env::temp_dir().to_str() {
                    None => Err("Failed to convert temp_dir"),
                    Some(tmp) => Ok(tmp.to_string())
                }
            })
            .unwrap_or("/tmp".to_string());
        PathBuf::from(tmp)
    }

    fn socket_path(n: u8) -> PathBuf {
        Self::ipc_path().join(format!("discord-ipc-{}", n))
    }
}

impl Connection for UnixConnection {
    fn connect() -> Result<Self> {
        let connection_name = Self::socket_path(0);
        let socket = UnixStream::connect(connection_name)?;
        socket.set_write_timeout(Some(time::Duration::from_secs(30)))?;
        socket.set_read_timeout(Some(time::Duration::from_secs(30)))?;
        Ok(Self { socket })
    }

    fn send<T>(&mut self, opcode: u32, payload: T) -> Result<()>
        where T: Payload + Debug
    {
        debug!("payload: {:#?}", payload);
        match Message::new(opcode, payload).encode() {
            Err(why) => error!("{:?}", why),
            Ok(bytes) => {
                self.socket.write_all(bytes.as_ref())?;
                debug!("sent opcode: {}", opcode);
                self.recv()?;
            }
        };
        Ok(())
    }

    fn recv(&mut self) -> Result<Vec<u8>> {
        let mut buf: Vec<u8> = Vec::with_capacity(1024);
        self.socket.read(buf.as_mut_slice())?;
        debug!("{:?}", buf);
        Ok(buf)
    }
}

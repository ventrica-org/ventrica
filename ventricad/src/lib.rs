pub mod ops;
pub mod protocol;

pub use protocol::{DEFAULT_SOCKET, Message, Request, SOCKET_ENV};
pub use ventrica::repo::PackageEntry;
pub use ventrica::{GenerationRecord, Package, PackageRecord, RepoRecord};

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

pub struct DaemonClient {
    writer: UnixStream,
    reader: BufReader<UnixStream>,
}

impl DaemonClient {
    pub fn connect() -> std::io::Result<Self> {
        let path = std::env::var(SOCKET_ENV).unwrap_or_else(|_| DEFAULT_SOCKET.to_owned());
        Self::connect_to(&path)
    }

    pub fn connect_to(path: &str) -> std::io::Result<Self> {
        let stream = UnixStream::connect(path)?;
        let reader_stream = stream.try_clone()?;
        Ok(Self {
            writer: stream,
            reader: BufReader::new(reader_stream),
        })
    }

    pub fn send<F>(&mut self, req: &Request, mut on_msg: F) -> std::io::Result<()>
    where
        F: FnMut(Message),
    {
        let mut line = serde_json::to_string(req)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        line.push('\n');
        self.writer.write_all(line.as_bytes())?;
        self.writer.flush()?;

        loop {
            let mut raw = String::new();
            let n = self.reader.read_line(&mut raw)?;
            if n == 0 {
                break;
            }
            let raw = raw.trim();
            if raw.is_empty() {
                continue;
            }
            let msg: Message = serde_json::from_str(raw)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            if matches!(msg, Message::Done) {
                break;
            }
            on_msg(msg);
        }
        Ok(())
    }
}

//! Unix domain socket implementation for IPC

use super::{IpcConfig, IpcConnection, IpcServer};
use anyhow::{Context, Result};
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::net::{UnixListener as StdUnixListener, UnixStream as StdUnixStream};
use std::path::PathBuf;
use std::time::Duration;

/// Unix domain socket stream
pub struct IpcStream {
    stream: StdUnixStream,
    id: String,
}

impl IpcStream {
    fn new(stream: StdUnixStream) -> Self {
        let id = format!("unix-{:p}", &stream);
        Self { stream, id }
    }
}

impl Read for IpcStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream.read(buf)
    }
}

impl Write for IpcStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stream.flush()
    }
}

impl IpcConnection for IpcStream {
    fn is_connected(&self) -> bool {
        self.stream.peer_addr().is_ok()
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn shutdown(&mut self) -> Result<()> {
        self.stream
            .shutdown(std::net::Shutdown::Both)
            .context("Failed to shutdown Unix socket")
    }
}

unsafe impl Send for IpcStream {}

/// Unix domain socket listener
pub struct IpcListener {
    listener: StdUnixListener,
    path: PathBuf,
}

impl IpcListener {
    pub fn new(name: &str, _config: &IpcConfig) -> Result<Self> {
        let path = if name.starts_with('/') {
            PathBuf::from(name)
        } else {
            crate::current_platform()
                .runtime_dir()?
                .join(format!("{}.sock", name))
        };

        // Remove existing socket file if it exists
        if path.exists() {
            std::fs::remove_file(&path).ok();
        }

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        let listener = StdUnixListener::bind(&path)
            .with_context(|| format!("Failed to bind Unix socket: {:?}", path))?;

        // Set permissions to 600 (owner read/write only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(&path)?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600);
            std::fs::set_permissions(&path, permissions)?;
        }

        Ok(Self { listener, path })
    }
}

impl IpcServer for IpcListener {
    type Stream = IpcStream;

    fn accept(&self) -> Result<Self::Stream> {
        let (stream, _addr) = self
            .listener
            .accept()
            .context("Failed to accept Unix socket connection")?;
        Ok(IpcStream::new(stream))
    }

    fn address(&self) -> String {
        self.path.display().to_string()
    }

    fn shutdown(&mut self) -> Result<()> {
        // Remove the socket file
        if self.path.exists() {
            std::fs::remove_file(&self.path).context("Failed to remove Unix socket file")?;
        }
        Ok(())
    }
}

impl Drop for IpcListener {
    fn drop(&mut self) {
        self.shutdown().ok();
    }
}

/// Unix domain socket client
pub struct IpcClient;

impl IpcClient {
    pub fn connect(name: &str, config: &IpcConfig) -> Result<IpcStream> {
        let path = if name.starts_with('/') {
            PathBuf::from(name)
        } else {
            crate::current_platform()
                .runtime_dir()?
                .join(format!("{}.sock", name))
        };

        // Note: UnixStream doesn't have connect_timeout in std, we'll use regular connect
        // and set timeouts after connection
        let stream = StdUnixStream::connect(&path)
            .with_context(|| format!("Failed to connect to Unix socket: {:?}", path))?;

        // Set timeouts if configured
        if let Some(timeout) = config.read_timeout {
            stream.set_read_timeout(Some(Duration::from_millis(timeout)))?;
        }
        if let Some(timeout) = config.write_timeout {
            stream.set_write_timeout(Some(Duration::from_millis(timeout)))?;
        }

        Ok(IpcStream::new(stream))
    }
}

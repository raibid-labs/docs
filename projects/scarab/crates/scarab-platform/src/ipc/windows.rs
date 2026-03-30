//! Windows Named Pipes implementation for IPC

use super::{IpcConfig, IpcConnection, IpcServer};
use anyhow::{anyhow, Context, Result};
use std::ffi::CString;
use std::io::{Read, Write};
use std::ptr::null_mut;
use std::time::Duration;
use winapi::shared::minwindef::{DWORD, FALSE, LPVOID, TRUE};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::fileapi::{CreateFileA, FlushFileBuffers, ReadFile, WriteFile, OPEN_EXISTING};
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::ioapiset::GetOverlappedResult;
use winapi::um::namedpipeapi::{
    ConnectNamedPipe, CreateNamedPipeA, DisconnectNamedPipe, SetNamedPipeHandleState,
    PIPE_ACCESS_DUPLEX, PIPE_NOWAIT, PIPE_READMODE_BYTE, PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES,
    PIPE_WAIT,
};
use winapi::um::synchapi::WaitForSingleObject;
use winapi::um::winbase::{FILE_FLAG_OVERLAPPED, INFINITE, WAIT_OBJECT_0};
use winapi::um::winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE, HANDLE};

/// Windows Named Pipe stream
pub struct IpcStream {
    handle: HANDLE,
    id: String,
    is_server: bool,
}

impl IpcStream {
    fn new(handle: HANDLE, is_server: bool) -> Self {
        let id = format!("pipe-{:p}", handle);
        Self {
            handle,
            id,
            is_server,
        }
    }
}

impl Read for IpcStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut bytes_read: DWORD = 0;
        let result = unsafe {
            ReadFile(
                self.handle,
                buf.as_mut_ptr() as LPVOID,
                buf.len() as DWORD,
                &mut bytes_read,
                null_mut(),
            )
        };

        if result == FALSE {
            let error = unsafe { GetLastError() };
            return Err(std::io::Error::from_raw_os_error(error as i32));
        }

        Ok(bytes_read as usize)
    }
}

impl Write for IpcStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut bytes_written: DWORD = 0;
        let result = unsafe {
            WriteFile(
                self.handle,
                buf.as_ptr() as LPVOID,
                buf.len() as DWORD,
                &mut bytes_written,
                null_mut(),
            )
        };

        if result == FALSE {
            let error = unsafe { GetLastError() };
            return Err(std::io::Error::from_raw_os_error(error as i32));
        }

        Ok(bytes_written as usize)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let result = unsafe { FlushFileBuffers(self.handle) };
        if result == FALSE {
            let error = unsafe { GetLastError() };
            return Err(std::io::Error::from_raw_os_error(error as i32));
        }
        Ok(())
    }
}

impl IpcConnection for IpcStream {
    fn is_connected(&self) -> bool {
        self.handle != INVALID_HANDLE_VALUE
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn shutdown(&mut self) -> Result<()> {
        if self.handle != INVALID_HANDLE_VALUE {
            if self.is_server {
                unsafe {
                    DisconnectNamedPipe(self.handle);
                }
            }
            unsafe {
                CloseHandle(self.handle);
            }
            self.handle = INVALID_HANDLE_VALUE;
        }
        Ok(())
    }
}

impl Drop for IpcStream {
    fn drop(&mut self) {
        self.shutdown().ok();
    }
}

unsafe impl Send for IpcStream {}

/// Windows Named Pipe listener
pub struct IpcListener {
    handle: HANDLE,
    name: String,
    config: IpcConfig,
}

impl IpcListener {
    pub fn new(name: &str, config: &IpcConfig) -> Result<Self> {
        let pipe_name = if name.starts_with(r"\\.\pipe\") {
            name.to_string()
        } else {
            format!(r"\\.\pipe\{}", name)
        };

        let pipe_name_cstring =
            CString::new(pipe_name.clone()).context("Failed to create CString for pipe name")?;

        let handle = unsafe {
            CreateNamedPipeA(
                pipe_name_cstring.as_ptr(),
                PIPE_ACCESS_DUPLEX,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                config.max_connections,
                config.buffer_size as DWORD,
                config.buffer_size as DWORD,
                0, // Default timeout
                null_mut(),
            )
        };

        if handle == INVALID_HANDLE_VALUE {
            let error = unsafe { GetLastError() };
            return Err(anyhow!(
                "Failed to create named pipe '{}': Error {}",
                pipe_name,
                error
            ));
        }

        Ok(Self {
            handle,
            name: pipe_name,
            config: config.clone(),
        })
    }

    fn create_new_instance(&self) -> Result<HANDLE> {
        let pipe_name_cstring =
            CString::new(self.name.clone()).context("Failed to create CString for pipe name")?;

        let handle = unsafe {
            CreateNamedPipeA(
                pipe_name_cstring.as_ptr(),
                PIPE_ACCESS_DUPLEX,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                self.config.max_connections,
                self.config.buffer_size as DWORD,
                self.config.buffer_size as DWORD,
                0,
                null_mut(),
            )
        };

        if handle == INVALID_HANDLE_VALUE {
            let error = unsafe { GetLastError() };
            return Err(anyhow!(
                "Failed to create new pipe instance: Error {}",
                error
            ));
        }

        Ok(handle)
    }
}

impl IpcServer for IpcListener {
    type Stream = IpcStream;

    fn accept(&self) -> Result<Self::Stream> {
        // Wait for a client to connect
        let result = unsafe { ConnectNamedPipe(self.handle, null_mut()) };

        if result == FALSE {
            let error = unsafe { GetLastError() };
            // ERROR_PIPE_CONNECTED (535) means a client is already connected
            if error != 535 {
                return Err(anyhow!("Failed to accept connection: Error {}", error));
            }
        }

        // Create a new instance for the next client
        let current_handle = self.handle;
        let new_handle = self.create_new_instance()?;

        // Return the connected stream using the current handle
        // Note: In a real implementation, we'd need to manage multiple instances
        Ok(IpcStream::new(current_handle, true))
    }

    fn address(&self) -> String {
        self.name.clone()
    }

    fn shutdown(&mut self) -> Result<()> {
        if self.handle != INVALID_HANDLE_VALUE {
            unsafe {
                DisconnectNamedPipe(self.handle);
                CloseHandle(self.handle);
            }
            self.handle = INVALID_HANDLE_VALUE;
        }
        Ok(())
    }
}

impl Drop for IpcListener {
    fn drop(&mut self) {
        self.shutdown().ok();
    }
}

/// Windows Named Pipe client
pub struct IpcClient;

impl IpcClient {
    pub fn connect(name: &str, config: &IpcConfig) -> Result<IpcStream> {
        let pipe_name = if name.starts_with(r"\\.\pipe\") {
            name.to_string()
        } else {
            format!(r"\\.\pipe\{}", name)
        };

        let pipe_name_cstring =
            CString::new(pipe_name.clone()).context("Failed to create CString for pipe name")?;

        // Try to connect with timeout
        let start = std::time::Instant::now();
        let timeout = Duration::from_millis(config.connect_timeout);

        loop {
            let handle = unsafe {
                CreateFileA(
                    pipe_name_cstring.as_ptr(),
                    GENERIC_READ | GENERIC_WRITE,
                    0, // No sharing
                    null_mut(),
                    OPEN_EXISTING,
                    0, // No special attributes
                    null_mut(),
                )
            };

            if handle != INVALID_HANDLE_VALUE {
                // Set pipe to byte mode
                let mut mode = PIPE_READMODE_BYTE | PIPE_WAIT;
                let result =
                    unsafe { SetNamedPipeHandleState(handle, &mut mode, null_mut(), null_mut()) };

                if result == FALSE {
                    unsafe {
                        CloseHandle(handle);
                    }
                    let error = unsafe { GetLastError() };
                    return Err(anyhow!("Failed to set pipe mode: Error {}", error));
                }

                return Ok(IpcStream::new(handle, false));
            }

            let error = unsafe { GetLastError() };

            // ERROR_PIPE_BUSY (231) means the pipe exists but is busy
            if error != 231 {
                return Err(anyhow!(
                    "Failed to connect to named pipe '{}': Error {}",
                    pipe_name,
                    error
                ));
            }

            // Check timeout
            if start.elapsed() >= timeout {
                return Err(anyhow!("Timeout connecting to named pipe '{}'", pipe_name));
            }

            // Wait a bit before retrying
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}

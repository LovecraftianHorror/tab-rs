mod child;
mod internal;
mod pty_file;

use async_trait::async_trait;

use std::{io, pin::Pin, sync::Arc};

use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::Mutex,
};

use crate::{Master, PtySystem};

use self::{child::UnixPtyChild, internal::UnixInternal};

pub struct UnixPtySystem {}

impl PtySystem for UnixPtySystem {
    type Child = UnixPtyChild;
    type Master = UnixPtyMaster;
    type MasterRead = UnixPtyRead;
    type MasterWrite = UnixPtyWrite;

    fn spawn(
        command: std::process::Command,
    ) -> Result<crate::PtySystemInstance<Self>, crate::PtySystemError> {
        todo!()
    }
}

pub struct UnixPtyMaster(Arc<UnixInternal>);

#[async_trait]
impl Master for UnixPtyMaster {
    async fn size(&self) -> std::io::Result<crate::Size> {
        todo!()
    }

    async fn resize(&self, size: crate::Size) -> std::io::Result<()> {
        todo!()
    }
}

pub struct UnixPtyRead(Arc<Mutex<UnixInternal>>);

impl AsyncRead for UnixPtyRead {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<io::Result<usize>> {
        AsyncRead::poll_read(Pin::new(&mut self.0), cx, buf)
    }
}

pub struct UnixPtyWrite(Arc<UnixInternal>);

impl AsyncWrite for UnixPtyWrite {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, io::Error>> {
        AsyncWrite::poll_write(Pin::new(&mut self.0), cx, buf)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), io::Error>> {
        AsyncWrite::poll_flush(Pin::new(&mut self.0), cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), io::Error>> {
        AsyncWrite::poll_shutdown(Pin::new(&mut self.0), cx)
    }
}

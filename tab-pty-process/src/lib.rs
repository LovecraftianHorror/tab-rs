mod unix;

use std::{io, process::ExitStatus};

use async_trait::async_trait;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};

#[derive(Error, Debug)]
pub enum PtySystemError {
    #[error("io error: {0}")]
    IoError(io::Error),
}

pub struct Size {
    pub width: u16,
    pub height: u16,
}

#[async_trait]
pub trait Child {
    async fn wait(self) -> io::Result<ExitStatus>;

    async fn kill(&mut self) -> io::Result<()>;
}

#[async_trait]
pub trait Master {
    async fn size(&self) -> io::Result<Size>;
    async fn resize(&self, size: Size) -> io::Result<()>;
}

pub trait PtySystem {
    type Child: Child;
    type Master: Master;
    type MasterRead: AsyncRead;
    type MasterWrite: AsyncWrite;

    fn spawn(command: std::process::Command) -> Result<PtySystemInstance<Self>, PtySystemError>;
}

pub struct PtySystemInstance<P>
where
    P: PtySystem + ?Sized,
{
    pub child: P::Child,
    pub master: P::Master,
    pub read: P::MasterRead,
    pub write: P::MasterWrite,
}

pub trait MasterRead: AsyncRead {}

pub trait MasterWrite: AsyncWrite {}

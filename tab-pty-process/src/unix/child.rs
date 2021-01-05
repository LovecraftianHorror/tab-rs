use std::{io, process::ExitStatus};

use tokio::process;

use crate::Child;
use async_trait::async_trait;

/// A child process that can be interacted with through a pseudo-TTY.
pub struct UnixPtyChild(process::Child);

impl UnixPtyChild {
    fn new(inner: process::Child) -> Self {
        Self(inner)
    }

    // Returns the OS-assigned process identifier associated with this child.
    // pub fn id(&self) -> u32 {
    //     self.0.id()
    // }
}

#[async_trait]
impl Child for UnixPtyChild {
    async fn wait(mut self) -> io::Result<ExitStatus> {
        process::Child::wait(&mut self.0).await
    }

    async fn kill(&mut self) -> io::Result<()> {
        process::Child::kill(&mut self.0).await
    }
}

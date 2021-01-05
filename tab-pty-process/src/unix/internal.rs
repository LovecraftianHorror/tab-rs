use std::{
    ffi::{CStr, OsStr},
    fs::{File, OpenOptions},
    io,
    os::unix::prelude::{FromRawFd, OsStrExt, RawFd},
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{unix::AsyncFd, AsyncRead, AsyncWrite};
pub struct UnixInternal {
    handle: AsyncFd<File>,
}

impl UnixInternal {
    pub fn new() -> io::Result<Self> {
        let inner = unsafe {
            // On MacOS, O_NONBLOCK is not documented as an allowed option to
            // posix_openpt(), but it is in fact allowed and functional, and
            // trying to add it later with fcntl() is forbidden. Meanwhile, on
            // FreeBSD, O_NONBLOCK is *not* an allowed option to
            // posix_openpt(), and the only way to get a nonblocking PTY
            // master is to add the nonblocking flag with fcntl() later. So,
            // we have to jump through some #[cfg()] hoops.

            const APPLY_NONBLOCK_AFTER_OPEN: bool = cfg!(target_os = "freebsd");

            let fd = if APPLY_NONBLOCK_AFTER_OPEN {
                libc::posix_openpt(libc::O_RDWR | libc::O_NOCTTY)
            } else {
                libc::posix_openpt(libc::O_RDWR | libc::O_NOCTTY | libc::O_NONBLOCK)
            };

            if fd < 0 {
                return Err(io::Error::last_os_error());
            }

            if libc::grantpt(fd) != 0 {
                return Err(io::Error::last_os_error());
            }

            if libc::unlockpt(fd) != 0 {
                return Err(io::Error::last_os_error());
            }

            if APPLY_NONBLOCK_AFTER_OPEN {
                let flags = libc::fcntl(fd, libc::F_GETFL, 0);
                if flags < 0 {
                    return Err(io::Error::last_os_error());
                }

                if libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) == -1 {
                    return Err(io::Error::last_os_error());
                }
            }

            File::from_raw_fd(fd)
        };

        let handle = AsyncFd::new(inner)?;

        Ok(Self { handle })
    }

    /// Open a pseudo-TTY slave that is connected to this master.
    ///
    /// The resulting file handle is *not* set to non-blocking mode.
    async fn open_sync_pty_slave(&self) -> Result<File, io::Error> {
        let mut buf: [libc::c_char; 512] = [0; 512];
        let fd = self.as_raw_fd();

        #[cfg(not(any(target_os = "macos", target_os = "freebsd")))]
        {
            if unsafe { libc::ptsname_r(fd, buf.as_mut_ptr(), buf.len()) } != 0 {
                return Err(io::Error::last_os_error());
            }
        }
        #[cfg(any(target_os = "macos", target_os = "freebsd"))]
        unsafe {
            let st = libc::ptsname(fd);
            if st.is_null() {
                return Err(io::Error::last_os_error());
            }
            libc::strncpy(buf.as_mut_ptr(), st, buf.len());
        }

        let ptsname = OsStr::from_bytes(unsafe { CStr::from_ptr(&buf as _) }.to_bytes());
        OpenOptions::new().read(true).write(true).open(ptsname)
    }

    pub async fn winsize(&self) -> std::io::Result<(u16, u16)> {
        let fd = self.as_raw_fd();
        let mut winsz: libc::winsize = unsafe { std::mem::zeroed() };

        if unsafe { libc::ioctl(fd, libc::TIOCGWINSZ.into(), &mut winsz) } != 0 {
            return Err(io::Error::last_os_error());
        }

        Ok((winsz.ws_col, winsz.ws_row))
    }

    pub async fn resize(&self, cols: u16, rows: u16) -> io::Result<()> {
        let fd = self.as_raw_fd();

        let winsz = libc::winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        if unsafe { libc::ioctl(fd, libc::TIOCSWINSZ.into(), &winsz) } != 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }

    pub(crate) fn poll_read_priv(
        &self,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<()>> {
        // Safety: `TcpStream::read` correctly handles reads into uninitialized memory
        unsafe { self.evented.x }
    }
}

impl AsyncRead for UnixInternal {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<io::Result<usize>> {
        AsyncRead::poll_read(Pin::new(&mut self.evented), cx, buf)
    }
}

impl AsyncWrite for UnixInternal {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, io::Error>> {
        AsyncWrite::poll_write(Pin::new(&mut self.evented), cx, buf)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), io::Error>> {
        AsyncWrite::poll_flush(Pin::new(&mut self.evented), cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), io::Error>> {
        AsyncWrite::poll_shutdown(Pin::new(&mut self.evented), cx)
    }
}

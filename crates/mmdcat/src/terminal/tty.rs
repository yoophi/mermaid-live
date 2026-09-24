use std::os::unix::io::RawFd;
use std::time::{Duration, Instant};

/// The controlling terminal, opened directly rather than through stdio.
///
/// The CLI is normally run from a shell widget whose stdin and stdout may be
/// redirected, so queries and image output must go to `/dev/tty` to reach the
/// terminal the user is actually looking at.
pub struct Tty {
    fd: RawFd,
    saved: Option<libc::termios>,
}

impl Tty {
    pub fn open() -> Option<Self> {
        // SAFETY: a C string literal and flags with no other preconditions.
        let fd = unsafe { libc::open(c"/dev/tty".as_ptr(), libc::O_RDWR) };
        if fd < 0 {
            return None;
        }

        Some(Self { fd, saved: None })
    }

    pub fn write(&self, data: &str) {
        let bytes = data.as_bytes();
        let mut written = 0;

        while written < bytes.len() {
            // SAFETY: writing `len - written` bytes from inside the slice.
            let count = unsafe {
                libc::write(
                    self.fd,
                    bytes[written..].as_ptr() as *const libc::c_void,
                    bytes.len() - written,
                )
            };

            if count <= 0 {
                return;
            }
            written += count as usize;
        }
    }

    /// The window size as the kernel knows it.
    ///
    /// Cheaper and more reliable than asking the terminal, though the pixel
    /// fields are zero on terminals that do not report them.
    pub fn window_size(&self) -> Option<libc::winsize> {
        // SAFETY: zeroed winsize is a valid target for TIOCGWINSZ.
        let mut size: libc::winsize = unsafe { std::mem::zeroed() };
        let result = unsafe { libc::ioctl(self.fd, libc::TIOCGWINSZ, &mut size) };

        if result < 0 {
            return None;
        }

        Some(size)
    }

    /// Sends a request and collects the reply until `is_complete` accepts it.
    ///
    /// Returns whatever arrived once the deadline passes, so a terminal that
    /// ignores the request costs one short timeout rather than hanging the
    /// shell. Bytes are mapped to chars one to one so escape sequences can be
    /// matched without worrying about UTF-8 boundaries.
    pub fn request<F>(&mut self, sequence: &str, is_complete: F, timeout: Duration) -> String
    where
        F: Fn(&str) -> bool,
    {
        if !self.enter_raw_mode() {
            return String::new();
        }

        self.write(sequence);

        let deadline = Instant::now() + timeout;
        let mut received = String::new();

        while Instant::now() < deadline {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match self.read_available(remaining) {
                Some(bytes) => {
                    received.extend(bytes.iter().map(|byte| *byte as char));
                    if is_complete(&received) {
                        break;
                    }
                }
                None => break,
            }
        }

        self.restore_mode();
        received
    }

    fn read_available(&self, timeout: Duration) -> Option<Vec<u8>> {
        let mut poll_fd = libc::pollfd {
            fd: self.fd,
            events: libc::POLLIN,
            revents: 0,
        };

        let timeout_ms = timeout.as_millis().min(i32::MAX as u128) as libc::c_int;
        // SAFETY: one initialised pollfd is passed with a matching count.
        let ready = unsafe { libc::poll(&mut poll_fd, 1, timeout_ms) };
        if ready <= 0 {
            return None;
        }

        let mut buffer = [0u8; 1024];
        // SAFETY: reading at most the buffer's length into the buffer.
        let count = unsafe {
            libc::read(
                self.fd,
                buffer.as_mut_ptr() as *mut libc::c_void,
                buffer.len(),
            )
        };

        if count <= 0 {
            return None;
        }

        Some(buffer[..count as usize].to_vec())
    }

    fn enter_raw_mode(&mut self) -> bool {
        // SAFETY: zeroed termios is a valid target for tcgetattr.
        let mut term: libc::termios = unsafe { std::mem::zeroed() };
        if unsafe { libc::tcgetattr(self.fd, &mut term) } < 0 {
            return false;
        }

        self.saved = Some(term);

        let mut raw = term;
        // SAFETY: `raw` is an initialised termios obtained above.
        unsafe {
            libc::cfmakeraw(&mut raw);
            if libc::tcsetattr(self.fd, libc::TCSANOW, &raw) < 0 {
                self.saved = None;
                return false;
            }
        }

        true
    }

    fn restore_mode(&mut self) {
        if let Some(saved) = self.saved.take() {
            // SAFETY: `saved` came from tcgetattr on this same descriptor.
            unsafe { libc::tcsetattr(self.fd, libc::TCSANOW, &saved) };
        }
    }
}

impl Drop for Tty {
    /// Restores the terminal even if a caller returned early.
    fn drop(&mut self) {
        self.restore_mode();
        // SAFETY: the descriptor was opened here and is closed once.
        unsafe { libc::close(self.fd) };
    }
}

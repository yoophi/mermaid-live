use std::collections::HashMap;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;

use crate::domain::chart_raster::RasterSpec;

/// The result the render window sends back for one request.
pub enum RenderReply {
    Png(Vec<u8>),
    Failed(String),
}

/// Renders waiting on their hidden window to report back.
///
/// Rasterising crosses from a worker thread into the webview and returns
/// through a Tauri command, so the two halves are matched up by request id.
#[derive(Default)]
pub struct PendingRenders {
    inner: Mutex<HashMap<String, Sender<RenderReply>>>,
}

impl PendingRenders {
    pub fn register(&self, request_id: String) -> Receiver<RenderReply> {
        let (sender, receiver) = channel();
        self.lock().insert(request_id, sender);
        receiver
    }

    /// Reports whether a render was actually waiting for this id.
    pub fn complete(&self, request_id: &str, reply: RenderReply) -> bool {
        let Some(sender) = self.lock().remove(request_id) else {
            return false;
        };

        // A closed receiver means the waiter already gave up.
        sender.send(reply).is_ok()
    }

    pub fn cancel(&self, request_id: &str) {
        self.lock().remove(request_id);
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, Sender<RenderReply>>> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;
    use std::os::unix::net::UnixStream;

    use super::{PendingRenders, RenderReply, Subscribers};
    use crate::domain::chart_raster::RasterSpec;

    fn spec() -> RasterSpec {
        RasterSpec::new(800, 600, 2.0).expect("valid spec")
    }

    #[test]
    fn delivers_a_line_to_a_subscriber() {
        let (ours, mut theirs) = UnixStream::pair().expect("a socket pair");
        let subscribers = Subscribers::default();
        let id = subscribers.register("req".to_string(), spec(), ours);

        assert!(subscribers.send(id, "hello\n"));

        let mut received = [0u8; 6];
        theirs.read_exact(&mut received).expect("reads the line");
        assert_eq!(&received, b"hello\n");
    }

    #[test]
    fn reports_who_to_render_for() {
        let (ours, _theirs) = UnixStream::pair().expect("a socket pair");
        let subscribers = Subscribers::default();
        assert!(subscribers.is_empty());

        let id = subscribers.register("req".to_string(), spec(), ours);

        assert_eq!(subscribers.targets(), vec![(id, "req".to_string(), spec())]);
        assert!(!subscribers.is_empty());
    }

    #[test]
    fn forgets_a_subscriber_that_went_away() {
        let (ours, theirs) = UnixStream::pair().expect("a socket pair");
        let subscribers = Subscribers::default();
        let id = subscribers.register("req".to_string(), spec(), ours);
        drop(theirs);

        // The first write may still be buffered; the second cannot be.
        let _ = subscribers.send(id, "one\n");
        let _ = subscribers.send(id, "two\n");

        assert!(subscribers.is_empty());
    }

    #[test]
    fn unregisters_on_request() {
        let (ours, _theirs) = UnixStream::pair().expect("a socket pair");
        let subscribers = Subscribers::default();
        let id = subscribers.register("req".to_string(), spec(), ours);

        subscribers.unregister(id);

        assert!(subscribers.is_empty());
        assert!(!subscribers.send(id, "gone\n"));
    }

    #[test]
    fn a_subscriber_gets_its_own_size() {
        let (one, _one_end) = UnixStream::pair().expect("a socket pair");
        let (two, _two_end) = UnixStream::pair().expect("a socket pair");
        let narrow = RasterSpec::new(400, 600, 1.0).expect("valid spec");
        let subscribers = Subscribers::default();

        subscribers.register("one".to_string(), spec(), one);
        subscribers.register("two".to_string(), narrow, two);

        let sizes: Vec<_> = subscribers.targets().into_iter().map(|(_, _, s)| s).collect();
        assert_eq!(sizes, vec![spec(), narrow]);
    }

    #[test]
    fn delivers_a_reply_to_the_waiting_render() {
        let pending = PendingRenders::default();
        let receiver = pending.register("one".to_string());

        assert!(pending.complete("one", RenderReply::Png(vec![1, 2, 3])));

        match receiver.recv().expect("a reply arrives") {
            RenderReply::Png(png) => assert_eq!(png, vec![1, 2, 3]),
            RenderReply::Failed(message) => panic!("unexpected failure: {message}"),
        }
    }

    #[test]
    fn reports_a_reply_for_an_unknown_request() {
        let pending = PendingRenders::default();

        assert!(!pending.complete("missing", RenderReply::Failed("nope".to_string())));
    }

    #[test]
    fn a_cancelled_render_no_longer_accepts_a_reply() {
        let pending = PendingRenders::default();
        let _receiver = pending.register("one".to_string());
        pending.cancel("one");

        assert!(!pending.complete("one", RenderReply::Png(vec![])));
    }
}

/// A terminal that asked to be shown every chart the app detects.
struct Subscriber {
    id: u64,
    /// The id the client used when subscribing, echoed back on every push.
    request_id: String,
    spec: RasterSpec,
    stream: UnixStream,
}

/// The terminals currently subscribed to detected charts.
///
/// Each one asked for its own image size, so a chart is rendered per distinct
/// size rather than once for everybody.
#[derive(Default)]
pub struct Subscribers {
    inner: Mutex<Vec<Subscriber>>,
    next_id: AtomicU64,
}

impl Subscribers {
    pub fn register(&self, request_id: String, spec: RasterSpec, stream: UnixStream) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.lock().push(Subscriber {
            id,
            request_id,
            spec,
            stream,
        });
        id
    }

    pub fn unregister(&self, id: u64) {
        self.lock().retain(|subscriber| subscriber.id != id);
    }

    pub fn is_empty(&self) -> bool {
        self.lock().is_empty()
    }

    /// Who to render for, at what size, and under which request id.
    pub fn targets(&self) -> Vec<(u64, String, RasterSpec)> {
        self.lock()
            .iter()
            .map(|subscriber| (subscriber.id, subscriber.request_id.clone(), subscriber.spec))
            .collect()
    }

    /// Sends one line, dropping the subscriber if the write fails.
    ///
    /// A failed write is how a terminal that went away announces itself, so it
    /// is the natural moment to forget it.
    pub fn send(&self, id: u64, line: &str) -> bool {
        let mut subscribers = self.lock();

        let Some(subscriber) = subscribers
            .iter_mut()
            .find(|subscriber| subscriber.id == id)
        else {
            return false;
        };

        let delivered = subscriber
            .stream
            .write_all(line.as_bytes())
            .and_then(|()| subscriber.stream.flush())
            .is_ok();

        if !delivered {
            subscribers.retain(|subscriber| subscriber.id != id);
        }

        delivered
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Vec<Subscriber>> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

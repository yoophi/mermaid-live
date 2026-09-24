use std::collections::VecDeque;
use std::sync::Mutex;

use crate::application::ports::{ChartRasterizer, ClipboardSource, DiagramAnalyzer};
use crate::application::raster_cache::RasterCache;
use crate::domain::chart_raster::{RasterError, RasterSpec};
use crate::domain::clipboard_chart::{ClipboardRevision, DetectedChart};
use crate::domain::diagram::{DiagramError, DiagramSource, DiagramValidation};
use crate::domain::mermaid_chart;

/// How many recently surfaced charts are remembered, so re-copying content the
/// user has already been shown does not surface it again.
const SEEN_CAPACITY: usize = 32;

/// Clipboard payloads far beyond any chart are dropped rather than scanned,
/// fingerprinted and staged. The watcher sees every copy the user makes, so a
/// large one must not turn into work.
const MAX_CLIPBOARD_TEXT_BYTES: usize = 512 * 1024;

pub struct ValidateDiagramSource<A>
where
    A: DiagramAnalyzer,
{
    analyzer: A,
}

impl<A> ValidateDiagramSource<A>
where
    A: DiagramAnalyzer,
{
    pub fn new(analyzer: A) -> Self {
        Self { analyzer }
    }

    pub fn execute(&self, raw_source: String) -> Result<DiagramValidation, DiagramError> {
        let source = DiagramSource::parse(raw_source)?;

        Ok(self.analyzer.analyze(&source))
    }
}

/// Decides whether the clipboard currently holds a Mermaid chart that the user
/// has not been shown yet.
pub struct DetectClipboardChart<C>
where
    C: ClipboardSource,
{
    clipboard: C,
    state: Mutex<DetectionState>,
}

#[derive(Default)]
struct DetectionState {
    last_revision: Option<ClipboardRevision>,
    seen: VecDeque<String>,
}

impl<C> DetectClipboardChart<C>
where
    C: ClipboardSource,
{
    pub fn new(clipboard: C) -> Self {
        Self {
            clipboard,
            state: Mutex::new(DetectionState::default()),
        }
    }

    /// Returns a chart only the first time it appears on the clipboard.
    ///
    /// The lock is held across the clipboard read so that the polling loop and
    /// an app activation happening at the same moment cannot both claim the
    /// same clipboard revision.
    pub fn execute(&self) -> Option<DetectedChart> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let revision = self.clipboard.revision();
        if revision.is_some() && revision == state.last_revision {
            return None;
        }
        state.last_revision = revision;

        let text = self.clipboard.read_text()?;
        if text.len() > MAX_CLIPBOARD_TEXT_BYTES {
            return None;
        }

        let source = mermaid_chart::extract_mermaid_chart_source(&text)?;
        let chart = DetectedChart::from_source(source);

        if state.seen.iter().any(|seen| seen == chart.fingerprint()) {
            return None;
        }

        state.seen.push_back(chart.fingerprint().to_string());
        while state.seen.len() > SEEN_CAPACITY {
            state.seen.pop_front();
        }

        Some(chart)
    }

    /// Records a chart as already surfaced.
    ///
    /// Used when the user has just been shown a chart by another route, such as
    /// rendering it in a terminal, so the watcher does not announce it again.
    pub fn mark_seen(&self, fingerprint: &str) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if state.seen.iter().any(|seen| seen == fingerprint) {
            return;
        }

        state.seen.push_back(fingerprint.to_string());
        while state.seen.len() > SEEN_CAPACITY {
            state.seen.pop_front();
        }
    }
}

/// What came of a request to render a chart for an external client.
#[derive(Debug, PartialEq, Eq)]
pub enum RenderChartOutcome {
    Rendered { fingerprint: String, png: Vec<u8> },
    /// The text was not a Mermaid chart. Not an error: callers hand over
    /// whatever the user selected.
    NotAChart,
}

/// Renders selected text as a chart image, reusing a recent render when the
/// same chart is asked for at the same size.
pub struct RenderChartToPng<'a, R>
where
    R: ChartRasterizer,
{
    rasterizer: R,
    cache: &'a RasterCache,
}

impl<'a, R> RenderChartToPng<'a, R>
where
    R: ChartRasterizer,
{
    pub fn new(rasterizer: R, cache: &'a RasterCache) -> Self {
        Self { rasterizer, cache }
    }

    pub fn execute(
        &self,
        text: &str,
        spec: RasterSpec,
    ) -> Result<RenderChartOutcome, RasterError> {
        if text.len() > MAX_CLIPBOARD_TEXT_BYTES {
            return Ok(RenderChartOutcome::NotAChart);
        }

        // The same extraction the clipboard watcher uses, so both routes agree
        // on what counts as a chart.
        let Some(source) = mermaid_chart::extract_mermaid_chart_source(text) else {
            return Ok(RenderChartOutcome::NotAChart);
        };

        let chart = DetectedChart::from_source(source);
        let key = spec.cache_key(chart.fingerprint());

        if let Some(png) = self.cache.get(&key) {
            return Ok(RenderChartOutcome::Rendered {
                fingerprint: chart.fingerprint().to_string(),
                png,
            });
        }

        let png = self.rasterizer.rasterize(&chart, spec)?;
        self.cache.put(key, png.clone());

        Ok(RenderChartOutcome::Rendered {
            fingerprint: chart.fingerprint().to_string(),
            png,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::DetectClipboardChart;
    use crate::application::ports::ClipboardSource;
    use crate::domain::clipboard_chart::ClipboardRevision;

    #[derive(Default)]
    struct FakeClipboard {
        state: Mutex<(Option<u64>, Option<String>)>,
    }

    impl FakeClipboard {
        fn copy(&self, revision: u64, text: &str) {
            *self.state.lock().unwrap() = (Some(revision), Some(text.to_string()));
        }

        fn conceal(&self, revision: u64) {
            *self.state.lock().unwrap() = (Some(revision), None);
        }
    }

    impl ClipboardSource for Arc<FakeClipboard> {
        fn revision(&self) -> Option<ClipboardRevision> {
            self.state.lock().unwrap().0.map(ClipboardRevision::new)
        }

        fn read_text(&self) -> Option<String> {
            self.state.lock().unwrap().1.clone()
        }
    }

    fn fixture() -> (Arc<FakeClipboard>, DetectClipboardChart<Arc<FakeClipboard>>) {
        let clipboard = Arc::new(FakeClipboard::default());
        let use_case = DetectClipboardChart::new(Arc::clone(&clipboard));
        (clipboard, use_case)
    }

    #[test]
    fn detects_a_new_chart_once_per_revision() {
        let (clipboard, use_case) = fixture();
        clipboard.copy(1, "flowchart LR\nA --> B");

        assert_eq!(
            use_case.execute().map(|chart| chart.label().to_string()),
            Some("flowchart LR".to_string())
        );
        assert!(use_case.execute().is_none());
    }

    #[test]
    fn ignores_content_the_user_was_already_shown() {
        let (clipboard, use_case) = fixture();
        clipboard.copy(1, "flowchart LR\nA --> B");
        assert!(use_case.execute().is_some());

        clipboard.copy(2, "flowchart LR\nA --> B");

        assert!(use_case.execute().is_none());
    }

    #[test]
    fn ignores_clipboard_text_that_is_not_a_chart() {
        let (clipboard, use_case) = fixture();
        clipboard.copy(1, "just some copied prose");

        assert!(use_case.execute().is_none());
    }

    #[test]
    fn ignores_concealed_clipboard_contents() {
        let (clipboard, use_case) = fixture();
        clipboard.conceal(1);

        assert!(use_case.execute().is_none());
    }

    #[test]
    fn ignores_clipboard_payloads_far_larger_than_a_chart() {
        let (clipboard, use_case) = fixture();
        let edge = "A --> B\n".repeat(super::MAX_CLIPBOARD_TEXT_BYTES / 8 + 1);
        clipboard.copy(1, &format!("flowchart LR\n{edge}"));

        assert!(use_case.execute().is_none());
    }

    #[test]
    fn marking_a_chart_seen_stops_it_being_announced() {
        let (clipboard, use_case) = fixture();
        use_case.mark_seen(&crate::domain::clipboard_chart::chart_fingerprint(
            "flowchart LR\nA --> B",
        ));

        clipboard.copy(1, "flowchart LR\nA --> B");

        assert!(use_case.execute().is_none());
    }

    #[test]
    fn detects_a_different_chart_after_the_first_one() {
        let (clipboard, use_case) = fixture();
        clipboard.copy(1, "flowchart LR\nA --> B");
        assert!(use_case.execute().is_some());

        clipboard.copy(2, "sequenceDiagram\nA->>B: hi");

        assert_eq!(
            use_case.execute().map(|chart| chart.label().to_string()),
            Some("sequenceDiagram".to_string())
        );
    }
}

#[cfg(test)]
mod render_tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::{RenderChartOutcome, RenderChartToPng};
    use crate::application::ports::ChartRasterizer;
    use crate::application::raster_cache::RasterCache;
    use crate::domain::chart_raster::{RasterError, RasterSpec};
    use crate::domain::clipboard_chart::DetectedChart;

    #[derive(Default)]
    struct CountingRasterizer {
        calls: AtomicUsize,
        fail_with: Option<String>,
    }

    impl ChartRasterizer for &CountingRasterizer {
        fn rasterize(
            &self,
            chart: &DetectedChart,
            _spec: RasterSpec,
        ) -> Result<Vec<u8>, RasterError> {
            self.calls.fetch_add(1, Ordering::Relaxed);

            match &self.fail_with {
                Some(message) => Err(RasterError::Renderer(message.clone())),
                None => Ok(chart.source().as_bytes().to_vec()),
            }
        }
    }

    fn spec() -> RasterSpec {
        RasterSpec::new(800, 600, 2.0).expect("valid spec")
    }

    #[test]
    fn renders_a_chart() {
        let rasterizer = CountingRasterizer::default();
        let cache = RasterCache::default();
        let use_case = RenderChartToPng::new(&rasterizer, &cache);

        let outcome = use_case
            .execute("flowchart LR\n  A --> B", spec())
            .expect("render succeeds");

        match outcome {
            RenderChartOutcome::Rendered { png, .. } => {
                assert_eq!(png, b"flowchart LR\n  A --> B".to_vec());
            }
            RenderChartOutcome::NotAChart => panic!("expected a rendered chart"),
        }
    }

    #[test]
    fn extracts_a_chart_out_of_a_fenced_code_block() {
        let rasterizer = CountingRasterizer::default();
        let cache = RasterCache::default();
        let use_case = RenderChartToPng::new(&rasterizer, &cache);

        let markdown = "설명\n\n```mermaid\nflowchart LR\n  A --> B\n```\n";
        let outcome = use_case.execute(markdown, spec()).expect("render succeeds");

        match outcome {
            RenderChartOutcome::Rendered { png, .. } => {
                assert_eq!(png, b"flowchart LR\n  A --> B".to_vec());
            }
            RenderChartOutcome::NotAChart => panic!("expected a rendered chart"),
        }
    }

    #[test]
    fn reports_text_that_is_not_a_chart_without_rendering() {
        let rasterizer = CountingRasterizer::default();
        let cache = RasterCache::default();
        let use_case = RenderChartToPng::new(&rasterizer, &cache);

        let outcome = use_case
            .execute("this text mentions graph but is not a diagram", spec())
            .expect("no rendering attempted");

        assert_eq!(outcome, RenderChartOutcome::NotAChart);
        assert_eq!(rasterizer.calls.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn ignores_payloads_far_larger_than_a_chart() {
        let rasterizer = CountingRasterizer::default();
        let cache = RasterCache::default();
        let use_case = RenderChartToPng::new(&rasterizer, &cache);

        let huge = format!(
            "flowchart LR\n{}",
            "A --> B\n".repeat(super::MAX_CLIPBOARD_TEXT_BYTES / 8 + 1)
        );

        assert_eq!(
            use_case.execute(&huge, spec()).expect("no rendering attempted"),
            RenderChartOutcome::NotAChart
        );
        assert_eq!(rasterizer.calls.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn renders_the_same_chart_only_once_per_size() {
        let rasterizer = CountingRasterizer::default();
        let cache = RasterCache::default();
        let use_case = RenderChartToPng::new(&rasterizer, &cache);

        use_case.execute("flowchart LR\n  A --> B", spec()).expect("first render");
        use_case.execute("flowchart LR\n  A --> B", spec()).expect("cached render");

        assert_eq!(rasterizer.calls.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn renders_again_when_the_requested_size_changes() {
        let rasterizer = CountingRasterizer::default();
        let cache = RasterCache::default();
        let use_case = RenderChartToPng::new(&rasterizer, &cache);

        use_case.execute("flowchart LR\n  A --> B", spec()).expect("first render");
        use_case
            .execute(
                "flowchart LR\n  A --> B",
                RasterSpec::new(400, 600, 2.0).expect("valid spec"),
            )
            .expect("second render");

        assert_eq!(rasterizer.calls.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn surfaces_renderer_failures() {
        let rasterizer = CountingRasterizer {
            calls: AtomicUsize::new(0),
            fail_with: Some("boom".to_string()),
        };
        let cache = RasterCache::default();
        let use_case = RenderChartToPng::new(&rasterizer, &cache);

        let error = use_case
            .execute("flowchart LR\n  A --> B", spec())
            .expect_err("renderer failed");

        assert!(error.to_string().contains("boom"));
    }
}

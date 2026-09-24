use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::domain::clipboard_chart::DetectedChart;

/// How many detected charts stay reachable from the tray menu.
const INBOX_CAPACITY: usize = 8;

/// A detected chart that has been written to disk and is ready to be shown.
#[derive(Clone, Debug)]
pub struct StagedChart {
    pub chart: DetectedChart,
    pub path: PathBuf,
    /// Whether a hidden window has already rendered this chart.
    pub prewarmed: bool,
    pub opened: bool,
}

/// Holds the charts detected while the user was working in another application.
#[derive(Default)]
pub struct ChartInbox {
    entries: Mutex<VecDeque<StagedChart>>,
}

impl ChartInbox {
    pub fn push(&self, staged: StagedChart) {
        self.with_entries(|entries| {
            entries.retain(|entry| entry.chart.fingerprint() != staged.chart.fingerprint());
            entries.push_back(staged);

            while entries.len() > INBOX_CAPACITY {
                entries.pop_front();
            }
        });
    }

    /// Every held chart, newest first.
    pub fn list(&self) -> Vec<StagedChart> {
        self.with_entries(|entries| entries.iter().rev().cloned().collect())
    }

    pub fn unopened_count(&self) -> usize {
        self.with_entries(|entries| entries.iter().filter(|entry| !entry.opened).count())
    }

    /// Marks the newest chart the user has not seen as opened, and returns it as
    /// it was before being claimed.
    pub fn claim_latest(&self) -> Option<StagedChart> {
        self.with_entries(|entries| claim(entries.iter_mut().rev().find(|entry| !entry.opened)?))
    }

    /// Marks one specific chart as opened, for the tray menu history.
    pub fn claim(&self, fingerprint: &str) -> Option<StagedChart> {
        self.with_entries(|entries| {
            claim(
                entries
                    .iter_mut()
                    .find(|entry| entry.chart.fingerprint() == fingerprint)?,
            )
        })
    }

    /// Releases the chart currently holding the pre-warmed window and returns
    /// its fingerprint, so the caller can close that window.
    pub fn release_prewarmed(&self) -> Option<String> {
        self.with_entries(|entries| {
            let entry = entries.iter_mut().rev().find(|entry| entry.prewarmed)?;
            entry.prewarmed = false;

            Some(entry.chart.fingerprint().to_string())
        })
    }

    fn with_entries<T>(&self, f: impl FnOnce(&mut VecDeque<StagedChart>) -> T) -> T {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        f(&mut entries)
    }
}

fn claim(entry: &mut StagedChart) -> Option<StagedChart> {
    let claimed = entry.clone();
    entry.opened = true;
    entry.prewarmed = false;

    Some(claimed)
}

#[cfg(test)]
mod tests {
    use super::{ChartInbox, StagedChart, INBOX_CAPACITY};
    use crate::domain::clipboard_chart::DetectedChart;

    fn staged(source: &str, prewarmed: bool) -> StagedChart {
        StagedChart {
            chart: DetectedChart::from_source(source.to_string()),
            path: std::path::PathBuf::from("/tmp/mermaid-live/test.mmd"),
            prewarmed,
            opened: false,
        }
    }

    #[test]
    fn claims_the_newest_unopened_chart_first() {
        let inbox = ChartInbox::default();
        inbox.push(staged("flowchart LR\nA --> B", false));
        inbox.push(staged("sequenceDiagram\nA->>B: hi", false));

        let claimed = inbox.claim_latest().expect("a chart to claim");

        assert_eq!(claimed.chart.label(), "sequenceDiagram");
        assert_eq!(inbox.unopened_count(), 1);
    }

    #[test]
    fn stops_counting_a_chart_once_it_is_claimed() {
        let inbox = ChartInbox::default();
        inbox.push(staged("flowchart LR\nA --> B", false));

        assert!(inbox.claim_latest().is_some());
        assert_eq!(inbox.unopened_count(), 0);
        assert!(inbox.claim_latest().is_none());
    }

    #[test]
    fn reports_the_prewarmed_state_as_it_was_before_claiming() {
        let inbox = ChartInbox::default();
        inbox.push(staged("flowchart LR\nA --> B", true));

        assert!(inbox.claim_latest().expect("a chart to claim").prewarmed);
        assert!(inbox.release_prewarmed().is_none());
    }

    #[test]
    fn releases_only_the_chart_holding_the_prewarmed_window() {
        let inbox = ChartInbox::default();
        inbox.push(staged("flowchart LR\nA --> B", false));
        inbox.push(staged("sequenceDiagram\nA->>B: hi", true));

        let released = inbox.release_prewarmed().expect("a prewarmed chart");

        assert_eq!(
            released,
            DetectedChart::from_source("sequenceDiagram\nA->>B: hi".to_string())
                .fingerprint()
                .to_string()
        );
        assert!(inbox.release_prewarmed().is_none());
    }

    #[test]
    fn keeps_a_re_pushed_chart_in_a_single_slot() {
        let inbox = ChartInbox::default();
        inbox.push(staged("flowchart LR\nA --> B", false));
        inbox.push(staged("flowchart LR\nA --> B", false));

        assert_eq!(inbox.list().len(), 1);
    }

    #[test]
    fn drops_the_oldest_chart_past_capacity() {
        let inbox = ChartInbox::default();
        let total = INBOX_CAPACITY + 2;
        for index in 0..total {
            inbox.push(staged(&format!("flowchart LR\nA --> B{index}"), false));
        }

        let entries = inbox.list();
        let newest = format!("flowchart LR\nA --> B{}", total - 1);

        assert_eq!(entries.len(), INBOX_CAPACITY);
        assert_eq!(
            entries.first().expect("newest entry").chart.source(),
            newest.as_str()
        );
    }
}

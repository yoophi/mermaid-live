use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

use crate::application::ports::ChartNotifier;
use crate::domain::clipboard_chart::DetectedChart;

/// Posts detected charts to the system notification centre.
///
/// Notifications are awareness only: the notification plugin cannot deliver
/// clicks on desktop, so the tray icon is what actually opens the chart.
pub struct SystemChartNotifier {
    app: AppHandle,
}

impl SystemChartNotifier {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl ChartNotifier for SystemChartNotifier {
    fn notify_detected(&self, chart: &DetectedChart, pending_count: usize) {
        let body = if pending_count > 1 {
            format!("{} · 대기 중 {pending_count}개", chart.label())
        } else {
            chart.label().to_string()
        };

        if let Err(error) = self
            .app
            .notification()
            .builder()
            .title("Mermaid 차트 감지")
            .body(body)
            .show()
        {
            eprintln!("[clipboard] failed to post notification: {error}");
        }
    }
}

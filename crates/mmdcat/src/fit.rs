use crate::png::PixelSize;
use crate::terminal::metrics::Metrics;

/// The desktop preview leaves 48px of padding around the diagram
/// (`FIT_PADDING` in mermaid-preview.tsx). One cell on each side is the
/// terminal equivalent at typical cell widths.
pub const FIT_PADDING_CELLS: u16 = 1;

/// Rows kept free so the image never pushes the prompt off screen.
pub const RESERVED_ROWS: u16 = 2;

pub const MIN_COLUMNS: u16 = 10;
pub const MIN_ROWS: u16 = 3;

/// Used only when neither the kernel nor the terminal reports a cell size.
const NOMINAL_CELL_WIDTH_PX: u32 = 8;
const NOMINAL_CELL_HEIGHT_PX: u32 = 17;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayArea {
    pub available_columns: u16,
    pub available_rows: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetBox {
    pub width_px: u32,
    pub height_px: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placement {
    pub columns: u16,
    /// `None` leaves the row count to the terminal, which preserves aspect ratio.
    pub rows: Option<u16>,
}

pub fn compute_display_area(metrics: &Metrics) -> DisplayArea {
    DisplayArea {
        available_columns: metrics
            .columns
            .saturating_sub(FIT_PADDING_CELLS * 2)
            .max(MIN_COLUMNS),
        available_rows: metrics.rows.saturating_sub(RESERVED_ROWS).max(MIN_ROWS),
    }
}

/// The pixel box the chart should be fitted into.
///
/// Mirrors the preview's fit rule: the chart is scaled down to fit both width
/// and height of the available area while keeping its aspect ratio. The scaling
/// itself happens during rasterisation, so the CLI only supplies the box.
pub fn compute_target_box(area: &DisplayArea, metrics: &Metrics, scale: f64) -> TargetBox {
    let cell_width = metrics.cell_width_px.unwrap_or(NOMINAL_CELL_WIDTH_PX);
    let cell_height = metrics.cell_height_px.unwrap_or(NOMINAL_CELL_HEIGHT_PX);

    TargetBox {
        width_px: ((f64::from(area.available_columns) * f64::from(cell_width) * scale).floor()
            as u32)
            .max(1),
        height_px: ((f64::from(area.available_rows) * f64::from(cell_height) * scale).floor()
            as u32)
            .max(1),
    }
}

fn cells_for(logical_px: f64, cell_px: u32, available: u16) -> u16 {
    let needed = (logical_px / f64::from(cell_px)).ceil().max(1.0);
    let needed = needed.min(f64::from(u16::MAX)) as u16;
    needed.min(available)
}

/// How many cells the rendered image should occupy.
///
/// When the cell size is unknown only the column count is given, and the
/// terminal computes the rows itself from the image aspect ratio.
pub fn compute_placement(
    image: PixelSize,
    metrics: &Metrics,
    scale: f64,
    area: &DisplayArea,
) -> Placement {
    let logical_width = f64::from(image.width) / scale;
    let logical_height = f64::from(image.height) / scale;

    match (metrics.cell_width_px, metrics.cell_height_px) {
        (Some(cell_width), Some(cell_height)) => Placement {
            columns: cells_for(logical_width, cell_width, area.available_columns),
            rows: Some(cells_for(
                logical_height,
                cell_height,
                area.available_rows,
            )),
        },
        _ => Placement {
            columns: cells_for(logical_width, NOMINAL_CELL_WIDTH_PX, area.available_columns),
            rows: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{compute_display_area, compute_placement, compute_target_box, DisplayArea, Placement};
    use crate::png::PixelSize;
    use crate::terminal::metrics::{CellSource, Metrics};

    fn metrics() -> Metrics {
        Metrics {
            columns: 100,
            rows: 30,
            cell_width_px: Some(8),
            cell_height_px: Some(17),
            cell_source: CellSource::CellQuery,
        }
    }

    #[test]
    fn leaves_padding_and_room_for_the_prompt() {
        assert_eq!(
            compute_display_area(&metrics()),
            DisplayArea {
                available_columns: 98,
                available_rows: 28
            }
        );
    }

    #[test]
    fn a_tiny_terminal_still_gets_a_usable_floor() {
        let area = compute_display_area(&Metrics {
            columns: 4,
            rows: 2,
            ..metrics()
        });

        assert!(area.available_columns >= super::MIN_COLUMNS);
        assert!(area.available_rows >= super::MIN_ROWS);
    }

    #[test]
    fn the_target_box_is_expressed_in_device_pixels() {
        let metrics = metrics();
        let area = compute_display_area(&metrics);
        let box_ = compute_target_box(&area, &metrics, 2.0);

        assert_eq!(box_.width_px, 98 * 8 * 2);
        assert_eq!(box_.height_px, 28 * 17 * 2);
    }

    #[test]
    fn placement_divides_the_image_back_down_by_the_scale() {
        let metrics = metrics();
        let area = compute_display_area(&metrics);
        let placement = compute_placement(
            PixelSize {
                width: 784 * 2,
                height: 476 * 2,
            },
            &metrics,
            2.0,
            &area,
        );

        assert_eq!(
            placement,
            Placement {
                columns: 98,
                rows: Some(28)
            }
        );
    }

    #[test]
    fn placement_never_exceeds_the_area_it_was_given() {
        let metrics = metrics();
        let area = compute_display_area(&metrics);
        let placement = compute_placement(
            PixelSize {
                width: 100_000,
                height: 100_000,
            },
            &metrics,
            2.0,
            &area,
        );

        assert_eq!(placement.columns, area.available_columns);
        assert_eq!(placement.rows, Some(area.available_rows));
    }

    #[test]
    fn the_row_count_is_left_out_when_the_cell_size_is_unknown() {
        let metrics = Metrics {
            cell_width_px: None,
            cell_height_px: None,
            cell_source: CellSource::Unavailable,
            ..metrics()
        };
        let area = compute_display_area(&metrics);
        let placement = compute_placement(
            PixelSize {
                width: 800,
                height: 400,
            },
            &metrics,
            1.0,
            &area,
        );

        assert_eq!(placement.rows, None);
    }
}

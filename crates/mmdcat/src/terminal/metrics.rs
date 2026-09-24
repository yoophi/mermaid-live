use std::time::Duration;

use super::tty::Tty;

/// Cell size in pixels. Precise, but less widely implemented.
const CELL_SIZE_QUERY: &str = "\x1b[16t";
/// Text area size in pixels. Widely supported, needs the cell grid to divide.
const TEXT_AREA_QUERY: &str = "\x1b[14t";
/// Text area size in cells, for when the kernel does not know it.
const GRID_QUERY: &str = "\x1b[18t";

const QUERY_TIMEOUT: Duration = Duration::from_millis(200);
const FALLBACK_COLUMNS: u16 = 80;
const FALLBACK_ROWS: u16 = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellSource {
    /// Read straight from the kernel, no terminal round trip.
    Ioctl,
    CellQuery,
    TextAreaQuery,
    Unavailable,
}

impl CellSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ioctl => "ioctl",
            Self::CellQuery => "cell-query",
            Self::TextAreaQuery => "text-area-query",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Metrics {
    pub columns: u16,
    pub rows: u16,
    /// `None` when neither the kernel nor the terminal reported a usable size.
    pub cell_width_px: Option<u32>,
    pub cell_height_px: Option<u32>,
    pub cell_source: CellSource,
}

/// Reads a `ESC [ <prefix> ; <first> ; <second> t` reply.
fn parse_pair(buffer: &str, prefix: char) -> Option<(u32, u32)> {
    let needle = format!("\x1b[{prefix};");
    let start = buffer.find(&needle)? + needle.len();
    let rest = &buffer[start..];
    let end = rest.find('t')?;

    let mut parts = rest[..end].split(';');
    let first = parts.next()?.trim().parse::<u32>().ok()?;
    let second = parts.next()?.trim().parse::<u32>().ok()?;

    if first == 0 || second == 0 {
        return None;
    }

    Some((first, second))
}

fn query_pair(tty: &mut Tty, sequence: &str, prefix: char) -> Option<(u32, u32)> {
    let reply = tty.request(
        sequence,
        |buffer| parse_pair(buffer, prefix).is_some(),
        QUERY_TIMEOUT,
    );

    parse_pair(&reply, prefix)
}

fn read_grid(tty: &mut Tty) -> (u16, u16) {
    if let Some(size) = tty.window_size() {
        if size.ws_col > 0 && size.ws_row > 0 {
            return (size.ws_col, size.ws_row);
        }
    }

    if let Some((rows, columns)) = query_pair(tty, GRID_QUERY, '8') {
        return (columns as u16, rows as u16);
    }

    (FALLBACK_COLUMNS, FALLBACK_ROWS)
}

/// Asks the kernel first, then the terminal, then gives up and reports the grid
/// alone so the caller can still size by width.
pub fn read_metrics(tty: Option<&mut Tty>) -> Metrics {
    let Some(tty) = tty else {
        return Metrics {
            columns: FALLBACK_COLUMNS,
            rows: FALLBACK_ROWS,
            cell_width_px: None,
            cell_height_px: None,
            cell_source: CellSource::Unavailable,
        };
    };

    let (columns, rows) = read_grid(tty);

    // Many terminals report zero for the pixel fields, so this is a fast path
    // rather than the only one.
    if let Some(size) = tty.window_size() {
        if size.ws_xpixel > 0 && size.ws_ypixel > 0 && columns > 0 && rows > 0 {
            return Metrics {
                columns,
                rows,
                cell_width_px: Some(u32::from(size.ws_xpixel) / u32::from(columns)),
                cell_height_px: Some(u32::from(size.ws_ypixel) / u32::from(rows)),
                cell_source: CellSource::Ioctl,
            };
        }
    }

    if let Some((height, width)) = query_pair(tty, CELL_SIZE_QUERY, '6') {
        return Metrics {
            columns,
            rows,
            cell_width_px: Some(width),
            cell_height_px: Some(height),
            cell_source: CellSource::CellQuery,
        };
    }

    if let Some((height, width)) = query_pair(tty, TEXT_AREA_QUERY, '4') {
        return Metrics {
            columns,
            rows,
            cell_width_px: Some(width / u32::from(columns)),
            cell_height_px: Some(height / u32::from(rows)),
            cell_source: CellSource::TextAreaQuery,
        };
    }

    Metrics {
        columns,
        rows,
        cell_width_px: None,
        cell_height_px: None,
        cell_source: CellSource::Unavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::parse_pair;

    #[test]
    fn reads_a_cell_size_reply() {
        assert_eq!(parse_pair("\x1b[6;17;8t", '6'), Some((17, 8)));
    }

    #[test]
    fn reads_a_reply_surrounded_by_noise() {
        assert_eq!(parse_pair("junk\x1b[4;800;1200tmore", '4'), Some((800, 1200)));
    }

    #[test]
    fn rejects_a_reply_for_another_query() {
        assert_eq!(parse_pair("\x1b[6;17;8t", '4'), None);
    }

    #[test]
    fn rejects_zero_and_malformed_replies() {
        assert_eq!(parse_pair("\x1b[6;0;8t", '6'), None);
        assert_eq!(parse_pair("\x1b[6;17t", '6'), None);
        assert_eq!(parse_pair("\x1b[6;17;8", '6'), None);
        assert_eq!(parse_pair("", '6'), None);
    }
}

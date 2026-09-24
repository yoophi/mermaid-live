use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;

use crate::fit::Placement;

const APC_START: &str = "\x1b_G";
const ST: &str = "\x1b\\";

/// The protocol caps a chunk at 4096 bytes and requires every chunk except the
/// last to be a multiple of 4. 4096 satisfies both.
pub const MAX_CHUNK_BYTES: usize = 4096;

fn placement_keys(placement: &Placement) -> String {
    match placement.rows {
        Some(rows) => format!("c={},r={}", placement.columns, rows),
        None => format!("c={}", placement.columns),
    }
}

/// Builds the escape sequences that transmit and display a PNG.
///
/// Uses direct transmission (`t=d`) because it is the one medium implemented by
/// every terminal this feature targets. `q=2` silences both success and error
/// replies, which a shell-driven client cannot consume.
pub fn build_image_sequences(png: &[u8], placement: &Placement) -> Vec<String> {
    let payload = BASE64.encode(png);
    if payload.is_empty() {
        return Vec::new();
    }

    let chunks: Vec<&str> = payload
        .as_bytes()
        .chunks(MAX_CHUNK_BYTES)
        // The payload is ASCII base64, so chunk boundaries are char boundaries.
        .map(|chunk| std::str::from_utf8(chunk).expect("base64 is ASCII"))
        .collect();

    let last = chunks.len() - 1;

    chunks
        .iter()
        .enumerate()
        .map(|(index, chunk)| {
            let more = if index == last { "m=0" } else { "m=1" };

            // Only the first escape code carries the full control set; the rest
            // may only repeat the chunking key.
            let controls = if index == 0 {
                format!(
                    "a=T,f=100,t=d,q=2,{},{more}",
                    placement_keys(placement)
                )
            } else {
                more.to_string()
            };

            format!("{APC_START}{controls};{chunk}{ST}")
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use base64::engine::general_purpose::STANDARD as BASE64;
    use base64::Engine;

    use super::{build_image_sequences, MAX_CHUNK_BYTES, ST};
    use crate::fit::Placement;

    fn payloads(sequences: &[String]) -> Vec<String> {
        sequences
            .iter()
            .map(|sequence| {
                let start = sequence.find(';').expect("has a payload separator") + 1;
                sequence[start..sequence.len() - ST.len()].to_string()
            })
            .collect()
    }

    #[test]
    fn a_small_image_is_one_transmit_and_display_sequence() {
        let sequences = build_image_sequences(
            b"tiny",
            &Placement {
                columns: 40,
                rows: Some(12),
            },
        );

        assert_eq!(sequences.len(), 1);
        let only = &sequences[0];
        assert!(only.starts_with("\x1b_G"));
        assert!(only.ends_with(ST));
        assert!(only.contains("a=T,f=100,t=d,q=2"));
        assert!(only.contains("c=40") && only.contains("r=12"));
        assert!(only.contains("m=0"));
    }

    #[test]
    fn the_row_count_is_left_out_when_it_is_unknown() {
        let sequences = build_image_sequences(
            b"tiny",
            &Placement {
                columns: 80,
                rows: None,
            },
        );

        assert!(sequences[0].contains("c=80"));
        assert!(!sequences[0].contains("r="));
    }

    #[test]
    fn chunking_follows_the_protocols_rules() {
        let png = vec![7u8; 12 * 1024];
        let sequences = build_image_sequences(
            &png,
            &Placement {
                columns: 80,
                rows: Some(20),
            },
        );
        let payloads = payloads(&sequences);

        let expected = BASE64.encode(&png).len().div_ceil(MAX_CHUNK_BYTES);
        assert_eq!(sequences.len(), expected);
        assert!(payloads.iter().all(|payload| payload.len() <= MAX_CHUNK_BYTES));
        // Every chunk but the last must be a multiple of 4.
        assert!(payloads[..payloads.len() - 1]
            .iter()
            .all(|payload| payload.len() % 4 == 0));
        // Only the first sequence carries the control set.
        assert!(sequences[1..]
            .iter()
            .all(|sequence| sequence.starts_with("\x1b_Gm=1;")
                || sequence.starts_with("\x1b_Gm=0;")));
        assert!(sequences.last().expect("has chunks").contains("m=0"));
        assert_eq!(
            BASE64.decode(payloads.concat()).expect("valid base64"),
            png
        );
    }

    #[test]
    fn an_empty_image_produces_nothing() {
        assert!(build_image_sequences(
            &[],
            &Placement {
                columns: 10,
                rows: None
            }
        )
        .is_empty());
    }
}

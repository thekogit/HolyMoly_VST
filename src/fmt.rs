//! Value text shared by the host (automation lanes) and the editor.
//! Matches `format` in reference/holy-moly-editor.html (plan.md C1b).

/// JavaScript `Math.round` (halves round up). Never returns -0.0.
fn js_round(x: f64) -> f64 {
    (x + 0.5).floor() + 0.0
}

pub fn percent(v: f32) -> String {
    let n = js_round(v as f64 * 100.0) as i64;
    format!("{n}%")
}

pub fn db(v: f32) -> String {
    with_unit(v, "dB")
}

pub fn semitones(v: f32) -> String {
    with_unit(v, "st")
}

fn with_unit(v: f32, unit: &str) -> String {
    let r = js_round(v as f64 * 10.0) / 10.0;
    let sign = if r > 0.0 { "+" } else { "" };
    format!("{sign}{r:.1} {unit}")
}

pub fn db_floor(v: f32, floor: f32) -> String {
    if v <= floor + 0.05 {
        "-inf dB".to_string()
    } else {
        db(v)
    }
}

/// Loop level: -60 dB and below prints as "-inf dB".
pub fn loop_level(v: f32) -> String {
    db_floor(v, -60.0)
}

pub fn hz(v: f32) -> String {
    if v >= 1000.0 {
        let k = v / 1000.0;
        format!("{k:.1} kHz")
    } else if v < 10.0 {
        format!("{v:.1} Hz")
    } else {
        let n = js_round(v as f64) as i64;
        format!("{n} Hz")
    }
}

pub fn ms(v: f32) -> String {
    if v >= 1000.0 {
        let s = v / 1000.0;
        format!("{s:.2} s")
    } else {
        let n = js_round(v as f64) as i64;
        format!("{n} ms")
    }
}

fn normalise(s: &str) -> String {
    s.trim().replace('\u{2212}', "-").to_lowercase()
}

/// The number at the start of `s`, e.g. "+6.0 dB" -> 6.0.
fn leading_number(s: &str) -> Option<f64> {
    let mut end = 0;
    for (i, c) in s.char_indices() {
        let ok = c.is_ascii_digit() || c == '.' || (i == 0 && (c == '-' || c == '+'));
        if !ok {
            break;
        }
        end = i + c.len_utf8();
    }
    s[..end].parse::<f64>().ok()
}

pub fn parse_percent(s: &str) -> Option<f32> {
    let s = normalise(s);
    leading_number(&s).map(|v| (v / 100.0) as f32)
}

pub fn parse_db(s: &str) -> Option<f32> {
    let s = normalise(s);
    leading_number(&s).map(|v| v as f32)
}

pub fn parse_semitones(s: &str) -> Option<f32> {
    parse_db(s)
}

pub fn parse_db_floor(s: &str, floor: f32) -> Option<f32> {
    let s = normalise(s);
    if s.contains("inf") {
        return Some(floor);
    }
    leading_number(&s).map(|v| v as f32)
}

pub fn parse_loop_level(s: &str) -> Option<f32> {
    parse_db_floor(s, -60.0)
}

pub fn parse_hz(s: &str) -> Option<f32> {
    let s = normalise(s);
    let v = leading_number(&s)?;
    let mult = if s.contains('k') { 1000.0 } else { 1.0 };
    Some((v * mult) as f32)
}

pub fn parse_ms(s: &str) -> Option<f32> {
    let s = normalise(s);
    let v = leading_number(&s)?;
    let mult = if s.contains("ms") || !s.ends_with('s') {
        1.0
    } else {
        1000.0
    };
    Some((v * mult) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Option<f32>, b: f32) -> bool {
        match a {
            Some(x) => (x - b).abs() <= 1e-4 * b.abs().max(1.0),
            None => false,
        }
    }

    #[test]
    fn percent_text() {
        assert_eq!(percent(0.5), "50%");
        assert_eq!(percent(0.0), "0%");
        assert_eq!(percent(1.1), "110%");
        assert_eq!(percent(0.35), "35%");
    }

    #[test]
    fn db_text() {
        assert_eq!(db(0.0), "0.0 dB");
        assert_eq!(db(6.0), "+6.0 dB");
        assert_eq!(db(-24.0), "-24.0 dB");
        assert_eq!(db(2.34), "+2.3 dB");
        assert_eq!(loop_level(-60.0), "-inf dB");
        assert_eq!(db_floor(-59.9, -60.0), "-59.9 dB");
    }

    #[test]
    fn hz_text() {
        assert_eq!(hz(440.0), "440 Hz");
        assert_eq!(hz(1500.0), "1.5 kHz");
        assert_eq!(hz(8000.0), "8.0 kHz");
        assert_eq!(hz(1.5), "1.5 Hz");
        assert_eq!(hz(20.0), "20 Hz");
    }

    #[test]
    fn ms_text() {
        assert_eq!(ms(400.0), "400 ms");
        assert_eq!(ms(1200.0), "1.20 s");
        assert_eq!(ms(10.0), "10 ms");
    }

    #[test]
    fn semitone_text() {
        assert_eq!(semitones(12.0), "+12.0 st");
        assert_eq!(semitones(-7.0), "-7.0 st");
        assert_eq!(semitones(0.0), "0.0 st");
    }

    #[test]
    fn parse_text() {
        assert!(close(parse_percent("50%"), 0.5));
        assert!(close(parse_percent(" 75 "), 0.75));
        assert!(parse_percent("abc").is_none());
        assert!(close(parse_db("+6.0 dB"), 6.0));
        assert!(close(parse_db("-3"), -3.0));
        assert!(close(parse_db("\u{2212}3 dB"), -3.0));
        assert!(close(parse_loop_level("-inf dB"), -60.0));
        assert!(close(parse_hz("1.5 kHz"), 1500.0));
        assert!(close(parse_hz("440 Hz"), 440.0));
        assert!(close(parse_ms("1.2 s"), 1200.0));
        assert!(close(parse_ms("400 ms"), 400.0));
        assert!(close(parse_ms("250"), 250.0));
        assert!(close(parse_semitones("+12.0 st"), 12.0));
    }
}

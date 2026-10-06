//! Pure layout maths (no egui), unit-tested. Mirrors the CSS flexbox rules of the reference.

/// `justify-content: space-around` with `gap`. Returns the left x of each item.
/// Negative free space or a single item behaves like `center` (CSS).
pub fn space_around(x0: f32, width: f32, items: &[f32], gap: f32) -> Vec<f32> {
    let n = items.len();
    if n == 0 {
        return Vec::new();
    }
    let used = items.iter().sum::<f32>() + gap * (n - 1) as f32;
    let free = width - used;
    let (start, extra) = if n == 1 || free < 0.0 {
        (x0 + free / 2.0, 0.0)
    } else {
        (x0 + free / (2.0 * n as f32), free / n as f32)
    };
    let mut out = Vec::with_capacity(n);
    let mut x = start;
    for w in items {
        out.push(x);
        x += w + gap + extra;
    }
    out
}

/// `justify-content: space-between` with `gap`. A single item sits at the start (CSS).
pub fn space_between(x0: f32, width: f32, items: &[f32], gap: f32) -> Vec<f32> {
    let n = items.len();
    if n == 0 {
        return Vec::new();
    }
    let used = items.iter().sum::<f32>() + gap * (n - 1) as f32;
    let extra = if n > 1 && width > used {
        (width - used) / (n - 1) as f32
    } else {
        0.0
    };
    let mut out = Vec::with_capacity(n);
    let mut x = x0;
    for w in items {
        out.push(x);
        x += w + gap + extra;
    }
    out
}

/// Centred vertical stack (`flex-direction: column; justify-content: center`). Returns top y of each row.
pub fn stack_center(y0: f32, height: f32, rows: &[f32], gap: f32) -> Vec<f32> {
    let n = rows.len();
    if n == 0 {
        return Vec::new();
    }
    let used = rows.iter().sum::<f32>() + gap * (n - 1) as f32;
    let mut y = y0 + (height - used) / 2.0;
    let mut out = Vec::with_capacity(n);
    for h in rows {
        out.push(y);
        y += h + gap;
    }
    out
}

/// Items placed left to right so that the last one ends at `right`. Returns the left x of each item.
pub fn right_run(right: f32, items: &[f32], gap: f32) -> Vec<f32> {
    let n = items.len();
    if n == 0 {
        return Vec::new();
    }
    let used = items.iter().sum::<f32>() + gap * (n - 1) as f32;
    let mut x = right - used;
    let mut out = Vec::with_capacity(n);
    for w in items {
        out.push(x);
        x += w + gap;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn space_around_matches_css() {
        assert_eq!(
            space_around(0.0, 100.0, &[20.0, 20.0], 0.0),
            vec![15.0, 65.0]
        );
        assert_eq!(
            space_around(0.0, 100.0, &[20.0, 20.0], 8.0),
            vec![13.0, 67.0]
        );
        assert_eq!(space_around(10.0, 100.0, &[40.0], 8.0), vec![40.0]);
        assert_eq!(
            space_around(0.0, 50.0, &[40.0, 40.0], 0.0),
            vec![-15.0, 25.0]
        );
    }

    #[test]
    fn space_between_matches_css() {
        assert_eq!(
            space_between(0.0, 100.0, &[20.0, 20.0], 0.0),
            vec![0.0, 80.0]
        );
        assert_eq!(
            space_between(0.0, 100.0, &[10.0, 10.0, 10.0], 0.0),
            vec![0.0, 45.0, 90.0]
        );
        assert_eq!(space_between(5.0, 100.0, &[10.0], 0.0), vec![5.0]);
    }

    #[test]
    fn stack_center_matches_css() {
        assert_eq!(
            stack_center(0.0, 184.0, &[84.0, 38.0], 8.0),
            vec![27.0, 119.0]
        );
    }

    #[test]
    fn right_run_ends_at_right() {
        assert_eq!(right_run(100.0, &[20.0, 30.0], 6.0), vec![44.0, 70.0]);
    }
}

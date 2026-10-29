use crate::storage::SessionRow;

/// Height in pixels of one bar. Always at least 2px so empty bars stay visible.
pub fn bar_height(value: u32, max: u32, max_px: f32) -> f32 {
    if max == 0 || value == 0 {
        return 2.0;
    }
    (value as f32 / max as f32 * max_px).max(2.0)
}

pub fn sort_longest(rows: &mut [SessionRow]) {
    rows.sort_by(|a, b| b.seconds.cmp(&a.seconds));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tallest_bar_uses_full_height() {
        assert_eq!(bar_height(100, 100, 200.0), 200.0);
    }

    #[test]
    fn half_value_is_half_height() {
        assert_eq!(bar_height(50, 100, 200.0), 100.0);
    }

    #[test]
    fn zero_values_do_not_crash() {
        assert_eq!(bar_height(0, 0, 200.0), 2.0);
    }

    #[test]
    fn sort_longest_puts_biggest_first() {
        let mut rows = vec![
            SessionRow { category: "Study".into(), seconds: 60 },
            SessionRow { category: "Exam".into(), seconds: 300 },
        ];
        sort_longest(&mut rows);
        assert_eq!(rows[0].category, "Exam");
    }
}

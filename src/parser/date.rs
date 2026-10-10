const CALENDARS: &[&str] = &["GREGORIAN", "JULIAN", "FRENCH_R", "HEBREW"];

fn is_epoch(tok: &str) -> bool {
    matches!(tok, "BCE" | "CE") || tok.starts_with('_')
}

fn is_calendar(tok: &str) -> bool {
    CALENDARS.contains(&tok) || tok.starts_with('_')
}

fn is_digits(tok: &str) -> bool {
    !tok.is_empty() && tok.bytes().all(|b| b.is_ascii_digit())
}

const MODIFIERS: &[&str] = &["ABT", "CAL", "EST", "AFT", "BEF", "TO", "AND"];

fn one_date(toks: &[&str]) -> Option<i32> {
    let mut start = 0usize;
    while start < toks.len() && is_calendar(toks[start]) {
        start += 1;
    }
    let mut end = toks.len();
    let mut bce = false;
    if end > start && is_epoch(toks[end - 1]) {
        bce = toks[end - 1] == "BCE";
        end -= 1;
    }
    let year = toks[start..end].iter().rev().find(|w| is_digits(w))?;
    let n: i32 = year.parse().ok()?;
    Some(if bce { -n } else { n })
}

fn after_modifier<'a>(all: &'a [&'a str], key: &str) -> Option<&'a [&'a str]> {
    let i = all.iter().position(|t| *t == key)?;
    Some(&all[i + 1..])
}

pub(crate) fn estimate_year(raw: &str) -> Option<i32> {
    let all: Vec<&str> = raw.split_whitespace().collect();
    if all.is_empty() {
        return Option::None;
    }
    if let Some(rest) = after_modifier(&all, "BET") {
        let Some(i) = rest.iter().position(|t| *t == "AND") else {
            return one_date(rest);
        };
        let first = one_date(&rest[..i])?;
        let last = one_date(&rest[i + 1..])?;
        return Some(if last == first {
            first
        } else {
            first + (last - first) / 2
        });
    }
    if let Some(rest) = after_modifier(&all, "FROM") {
        let end = rest.iter().position(|t| *t == "TO").unwrap_or(rest.len());
        return one_date(&rest[..end]);
    }
    for m in MODIFIERS {
        if let Some(rest) = after_modifier(&all, m) {
            return one_date(rest);
        }
    }
    one_date(&all)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn legacy(v: &str) -> Option<i32> {
        v.split_whitespace()
            .last()
            .filter(|y| y.len() >= 3 && y.bytes().all(|b| b.is_ascii_digit()))
            .and_then(|y| y.parse().ok())
    }

    #[test]
    fn plain_dates() {
        assert_eq!(estimate_year("1905"), Some(1905));
        assert_eq!(estimate_year("22 FEB 2026"), Some(2026));
        assert_eq!(estimate_year("1971"), Some(1971));
        assert_eq!(estimate_year("  1980  "), Some(1980));
        assert_eq!(estimate_year("12 MAR 1820"), Some(1820));
    }

    #[test]
    fn approximations_keep_stated_year() {
        for v in ["ABT 1973", "CAL 1973", "EST 1973", "BEF 1973", "AFT 1973"] {
            assert_eq!(estimate_year(v), Some(1973), "on {v}");
        }
        assert_eq!(estimate_year("ABT 12 MAR 1900"), Some(1900));
    }

    #[test]
    fn ranges_average() {
        assert_eq!(estimate_year("BET 1900 AND 1950"), Some(1925));
        assert_eq!(estimate_year("BET 1900 AND 1902"), Some(1901));
        assert_eq!(estimate_year("BET 1900 AND 1900"), Some(1900));
    }

    #[test]
    fn periods_use_start() {
        assert_eq!(estimate_year("FROM 667 BCE TO 324"), Some(-667));
        assert_eq!(estimate_year("FROM 1950 TO 1960"), Some(1950));
        assert_eq!(estimate_year("TO 1960"), Some(1960));
    }

    #[test]
    fn epochs_are_signed() {
        assert_eq!(estimate_year("1820 BCE"), Some(-1820));
        assert_eq!(estimate_year("12 MAR 500 BCE"), Some(-500));
        assert_eq!(estimate_year("500 CE"), Some(500));
        assert_eq!(estimate_year("FROM 500 BCE TO 600 CE"), Some(-500));
    }

    #[test]
    fn calendars_are_skipped() {
        assert_eq!(estimate_year("JULIAN 12 MAR 1500"), Some(1500));
        assert_eq!(estimate_year("HEBREW 5660"), Some(5660));
        assert_eq!(estimate_year("GREGORIAN 1999"), Some(1999));
    }

    #[test]
    fn junk_is_none() {
        for v in ["", "   ", "ABOUT", "unknown", "///"] {
            assert_eq!(estimate_year(v), None, "on {v:?}");
        }
    }

    #[test]
    fn matches_previous_behaviour_on_real_shapes() {
        for v in ["1905", "22 FEB 2026", "1971", "2002", "1999", "6 AUG 2025"] {
            assert_eq!(estimate_year(v), legacy(v), "regression on {v}");
        }
    }
}

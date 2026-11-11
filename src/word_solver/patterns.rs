//! Patterns that several families share (Python's module-level regex constants).

use crate::lazy_re;

/// `_SPELLED_ANY`: a number written as a word that would change the equation.
pub const SPELLED_ANY: &str = r"\b(?:two|three|four|five|six|seven|eight|nine|ten|twice|double|doubles|triple|half|halves|thrice)\b";
lazy_re!(pub SPELLED_ANY_RE, SPELLED_ANY);

// `_CHANGES_EQUATION`
lazy_re!(pub CHANGES_EQ_RE, r"\b(?:such|of the year|for part of|of the time|part of the time|some of the time)\b");

// `_EACH_AFTER`: "$200 each", "$200 apiece", "$5 for each".
lazy_re!(pub EACH_AFTER, r"\s*(?:(?:for|a)\s+)?(?:each|apiece|per|piece)\b");

// `re.search(r"\d", text)`
lazy_re!(pub DIGIT_RE, r"\d");

/// `_SPEED_UNIT` and the patterns built on it
pub const SPEED_UNIT: &str = r"mph|mi/h|miles per hour|miles an hour|miles/hour|km/h|kph|kmh|km/hr|kilometers per hour|kilometres per hour|km per hour";

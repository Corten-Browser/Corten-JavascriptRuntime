//! ECMAScript Internationalization API (ECMA-402) implementation
//!
//! This module implements the Intl namespace with:
//! - Collator: Locale-sensitive string comparison
//! - NumberFormat: Locale-sensitive number formatting
//! - DateTimeFormat: Locale-sensitive date/time formatting
//! - PluralRules: Plural-sensitive formatting
//! - RelativeTimeFormat: Relative time formatting
//! - ListFormat: Locale-sensitive list formatting
//!
//! # Example
//!
//! ```
//! use builtins::intl::{NumberFormat, NumberFormatOptions, Locale};
//!
//! let locale = Locale::new("en-US").unwrap();
//! let options = NumberFormatOptions::currency("USD");
//! let formatter = NumberFormat::new(locale, options);
//! assert_eq!(formatter.format(1234.56), "$1,234.56");
//! ```

use std::cmp::Ordering;
use std::collections::HashMap;

use crate::value::{JsError, JsResult};

// ============================================================================
// Locale
// ============================================================================

/// BCP 47 language tag representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locale {
    /// Primary language subtag (e.g., "en", "fr", "de")
    pub language: String,
    /// Optional script subtag (e.g., "Latn", "Hans")
    pub script: Option<String>,
    /// Optional region subtag (e.g., "US", "GB", "DE")
    pub region: Option<String>,
    /// Unicode extension keywords (e.g., "ca" for calendar, "nu" for numbering system)
    pub extensions: HashMap<String, String>,
}

impl Locale {
    /// Create a new Locale from a BCP 47 language tag string
    ///
    /// # Arguments
    /// * `tag` - A BCP 47 language tag (e.g., "en-US", "de-DE", "zh-Hans-CN")
    ///
    /// # Examples
    /// ```
    /// use builtins::intl::Locale;
    ///
    /// let locale = Locale::new("en-US").unwrap();
    /// assert_eq!(locale.language, "en");
    /// assert_eq!(locale.region, Some("US".to_string()));
    /// ```
    pub fn new(tag: &str) -> JsResult<Self> {
        let tag = tag.trim();
        if tag.is_empty() {
            return Err(JsError::range_error("Invalid language tag: empty string"));
        }

        let mut parts: Vec<&str> = tag.split('-').collect();
        let mut extensions = HashMap::new();

        // Parse language subtag (required)
        let language = parts.remove(0).to_lowercase();
        if !Self::is_valid_language(&language) {
            return Err(JsError::range_error(format!(
                "Invalid language subtag: {}",
                language
            )));
        }

        let mut script = None;
        let mut region = None;

        // Parse remaining subtags
        let mut i = 0;
        while i < parts.len() {
            let part = parts[i];

            // Check for Unicode extension
            if part == "u" && i + 1 < parts.len() {
                // Parse Unicode extension keywords
                i += 1;
                while i < parts.len() {
                    let key = parts[i];
                    if key.len() == 2 && key.chars().all(|c| c.is_ascii_alphanumeric()) {
                        if i + 1 < parts.len() && parts[i + 1].len() > 2 {
                            extensions.insert(key.to_string(), parts[i + 1].to_string());
                            i += 2;
                        } else {
                            extensions.insert(key.to_string(), "true".to_string());
                            i += 1;
                        }
                    } else {
                        break;
                    }
                }
                continue;
            }

            // Script subtag: 4 letters, title case
            if script.is_none() && part.len() == 4 && part.chars().all(|c| c.is_ascii_alphabetic()) {
                let mut chars = part.chars();
                let first = chars.next().unwrap().to_uppercase().to_string();
                let rest: String = chars.collect::<String>().to_lowercase();
                script = Some(format!("{}{}", first, rest));
                i += 1;
                continue;
            }

            // Region subtag: 2 letters (uppercase) or 3 digits
            if region.is_none() {
                if part.len() == 2 && part.chars().all(|c| c.is_ascii_alphabetic()) {
                    region = Some(part.to_uppercase());
                    i += 1;
                    continue;
                } else if part.len() == 3 && part.chars().all(|c| c.is_ascii_digit()) {
                    region = Some(part.to_string());
                    i += 1;
                    continue;
                }
            }

            // Skip unknown subtags
            i += 1;
        }

        Ok(Locale {
            language,
            script,
            region,
            extensions,
        })
    }

    /// Create a default locale (en-US)
    pub fn default() -> Self {
        Locale {
            language: "en".to_string(),
            script: None,
            region: Some("US".to_string()),
            extensions: HashMap::new(),
        }
    }

    /// Convert locale to BCP 47 tag string
    pub fn to_string(&self) -> String {
        let mut result = self.language.clone();

        if let Some(ref script) = self.script {
            result.push('-');
            result.push_str(script);
        }

        if let Some(ref region) = self.region {
            result.push('-');
            result.push_str(region);
        }

        if !self.extensions.is_empty() {
            result.push_str("-u");
            for (key, value) in &self.extensions {
                result.push('-');
                result.push_str(key);
                if value != "true" {
                    result.push('-');
                    result.push_str(value);
                }
            }
        }

        result
    }

    /// Check if a language subtag is valid (2-3 lowercase letters)
    fn is_valid_language(lang: &str) -> bool {
        (lang.len() == 2 || lang.len() == 3) && lang.chars().all(|c| c.is_ascii_lowercase())
    }

    /// Get the base name (without extensions)
    pub fn base_name(&self) -> String {
        let mut result = self.language.clone();
        if let Some(ref script) = self.script {
            result.push('-');
            result.push_str(script);
        }
        if let Some(ref region) = self.region {
            result.push('-');
            result.push_str(region);
        }
        result
    }

    /// Maximize the locale by adding likely subtags
    pub fn maximize(&self) -> Self {
        // Simplified implementation - would use CLDR likely subtags data in full impl
        let mut result = self.clone();

        // Add default script and region based on language
        if result.script.is_none() {
            result.script = match result.language.as_str() {
                "zh" => Some("Hans".to_string()),
                "ja" => Some("Jpan".to_string()),
                "ko" => Some("Kore".to_string()),
                _ => Some("Latn".to_string()),
            };
        }

        if result.region.is_none() {
            result.region = match result.language.as_str() {
                "en" => Some("US".to_string()),
                "zh" => Some("CN".to_string()),
                "ja" => Some("JP".to_string()),
                "ko" => Some("KR".to_string()),
                "de" => Some("DE".to_string()),
                "fr" => Some("FR".to_string()),
                "es" => Some("ES".to_string()),
                "it" => Some("IT".to_string()),
                "pt" => Some("BR".to_string()),
                "ru" => Some("RU".to_string()),
                "ar" => Some("SA".to_string()),
                _ => Some("001".to_string()), // World
            };
        }

        result
    }

    /// Minimize the locale by removing likely subtags
    pub fn minimize(&self) -> Self {
        // Simplified implementation
        Locale {
            language: self.language.clone(),
            script: None,
            region: None,
            extensions: self.extensions.clone(),
        }
    }
}

impl Default for Locale {
    fn default() -> Self {
        Locale::default()
    }
}

// ============================================================================
// Collator
// ============================================================================

/// Sensitivity options for collation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollatorSensitivity {
    /// Only strings that differ in base letters compare as unequal
    Base,
    /// Strings that differ in base letters or accents compare as unequal
    Accent,
    /// Strings that differ in base letters or case compare as unequal
    Case,
    /// Strings that differ in base letters, accents, case, or width compare as unequal (default)
    Variant,
}

impl Default for CollatorSensitivity {
    fn default() -> Self {
        CollatorSensitivity::Variant
    }
}

/// Case-first option for collation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseFirst {
    /// No special case ordering
    False,
    /// Upper case sorts before lower case
    Upper,
    /// Lower case sorts before upper case
    Lower,
}

impl Default for CaseFirst {
    fn default() -> Self {
        CaseFirst::False
    }
}

/// Collation usage type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollatorUsage {
    /// For sorting (default)
    Sort,
    /// For searching
    Search,
}

impl Default for CollatorUsage {
    fn default() -> Self {
        CollatorUsage::Sort
    }
}

/// Options for Collator
#[derive(Debug, Clone, Default)]
pub struct CollatorOptions {
    /// The usage type (sort or search)
    pub usage: CollatorUsage,
    /// The sensitivity level
    pub sensitivity: CollatorSensitivity,
    /// Whether to ignore punctuation
    pub ignore_punctuation: bool,
    /// Whether to use numeric collation
    pub numeric: bool,
    /// Case ordering
    pub case_first: CaseFirst,
}

/// Locale-sensitive string comparison
#[derive(Debug, Clone)]
pub struct Collator {
    locale: Locale,
    options: CollatorOptions,
}

impl Collator {
    /// Create a new Collator with the given locale and options
    pub fn new(locale: Locale, options: CollatorOptions) -> Self {
        Collator { locale, options }
    }

    /// Create a Collator with default options
    pub fn with_locale(locale: Locale) -> Self {
        Collator {
            locale,
            options: CollatorOptions::default(),
        }
    }

    /// Compare two strings according to the collation rules
    ///
    /// Returns:
    /// - `Ordering::Less` if x < y
    /// - `Ordering::Equal` if x == y
    /// - `Ordering::Greater` if x > y
    pub fn compare(&self, x: &str, y: &str) -> Ordering {
        let x_normalized = self.normalize(x);
        let y_normalized = self.normalize(y);

        if self.options.numeric {
            // Numeric collation: compare numeric substrings as numbers
            return self.compare_numeric(&x_normalized, &y_normalized);
        }

        match self.options.sensitivity {
            CollatorSensitivity::Base => {
                // Case-insensitive, accent-insensitive
                let x_base = self.to_base_form(&x_normalized);
                let y_base = self.to_base_form(&y_normalized);
                x_base.cmp(&y_base)
            }
            CollatorSensitivity::Accent => {
                // Case-insensitive, accent-sensitive
                let x_lower = x_normalized.to_lowercase();
                let y_lower = y_normalized.to_lowercase();
                x_lower.cmp(&y_lower)
            }
            CollatorSensitivity::Case => {
                // Case-sensitive, accent-insensitive
                let x_base = self.to_base_form(&x_normalized);
                let y_base = self.to_base_form(&y_normalized);
                let base_cmp = x_base.cmp(&y_base);
                if base_cmp != Ordering::Equal {
                    return base_cmp;
                }
                // Same base, compare with case
                x_normalized.cmp(&y_normalized)
            }
            CollatorSensitivity::Variant => {
                // Full comparison
                match self.options.case_first {
                    CaseFirst::Upper => {
                        // Compare with upper case first
                        self.compare_case_first(&x_normalized, &y_normalized, true)
                    }
                    CaseFirst::Lower => {
                        // Compare with lower case first
                        self.compare_case_first(&x_normalized, &y_normalized, false)
                    }
                    CaseFirst::False => x_normalized.cmp(&y_normalized),
                }
            }
        }
    }

    /// Get the locale used by this Collator
    pub fn resolved_options(&self) -> CollatorResolvedOptions {
        CollatorResolvedOptions {
            locale: self.locale.to_string(),
            usage: self.options.usage,
            sensitivity: self.options.sensitivity,
            ignore_punctuation: self.options.ignore_punctuation,
            numeric: self.options.numeric,
            case_first: self.options.case_first,
        }
    }

    /// Normalize the string (remove punctuation if configured)
    fn normalize(&self, s: &str) -> String {
        if self.options.ignore_punctuation {
            s.chars()
                .filter(|c| !c.is_ascii_punctuation())
                .collect()
        } else {
            s.to_string()
        }
    }

    /// Convert to base form (lowercase, no accents)
    fn to_base_form(&self, s: &str) -> String {
        s.to_lowercase()
            .chars()
            .map(|c| Self::remove_diacritics(c))
            .collect()
    }

    /// Remove diacritics from a character
    fn remove_diacritics(c: char) -> char {
        // Simplified diacritic removal - covers common Latin characters
        match c {
            'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' | 'ā' => 'a',
            'é' | 'è' | 'ê' | 'ë' | 'ē' => 'e',
            'í' | 'ì' | 'î' | 'ï' | 'ī' => 'i',
            'ó' | 'ò' | 'ô' | 'ö' | 'õ' | 'ō' => 'o',
            'ú' | 'ù' | 'û' | 'ü' | 'ū' => 'u',
            'ý' | 'ÿ' => 'y',
            'ñ' => 'n',
            'ç' => 'c',
            'ß' => 's',
            _ => c,
        }
    }

    /// Compare strings with numeric collation
    fn compare_numeric(&self, x: &str, y: &str) -> Ordering {
        let x_parts = Self::split_numeric(x);
        let y_parts = Self::split_numeric(y);

        for (x_part, y_part) in x_parts.iter().zip(y_parts.iter()) {
            let cmp = match (x_part, y_part) {
                (NumericPart::Text(a), NumericPart::Text(b)) => a.cmp(b),
                (NumericPart::Number(a), NumericPart::Number(b)) => a.cmp(b),
                (NumericPart::Text(_), NumericPart::Number(_)) => Ordering::Greater,
                (NumericPart::Number(_), NumericPart::Text(_)) => Ordering::Less,
            };
            if cmp != Ordering::Equal {
                return cmp;
            }
        }

        x_parts.len().cmp(&y_parts.len())
    }

    /// Split string into numeric and non-numeric parts
    fn split_numeric(s: &str) -> Vec<NumericPart> {
        let mut parts = Vec::new();
        let mut current_text = String::new();
        let mut current_num = String::new();

        for c in s.chars() {
            if c.is_ascii_digit() {
                if !current_text.is_empty() {
                    parts.push(NumericPart::Text(current_text.clone()));
                    current_text.clear();
                }
                current_num.push(c);
            } else {
                if !current_num.is_empty() {
                    if let Ok(n) = current_num.parse::<u64>() {
                        parts.push(NumericPart::Number(n));
                    }
                    current_num.clear();
                }
                current_text.push(c);
            }
        }

        if !current_text.is_empty() {
            parts.push(NumericPart::Text(current_text));
        }
        if !current_num.is_empty() {
            if let Ok(n) = current_num.parse::<u64>() {
                parts.push(NumericPart::Number(n));
            }
        }

        parts
    }

    /// Compare with case ordering preference
    fn compare_case_first(&self, x: &str, y: &str, upper_first: bool) -> Ordering {
        let x_lower = x.to_lowercase();
        let y_lower = y.to_lowercase();

        match x_lower.cmp(&y_lower) {
            Ordering::Equal => {
                // Same when case-insensitive, now compare case
                for (xc, yc) in x.chars().zip(y.chars()) {
                    let x_upper = xc.is_uppercase();
                    let y_upper = yc.is_uppercase();
                    if x_upper != y_upper {
                        if upper_first {
                            return if x_upper {
                                Ordering::Less
                            } else {
                                Ordering::Greater
                            };
                        } else {
                            return if x_upper {
                                Ordering::Greater
                            } else {
                                Ordering::Less
                            };
                        }
                    }
                }
                Ordering::Equal
            }
            other => other,
        }
    }
}

/// Helper enum for numeric collation
#[derive(Debug)]
enum NumericPart {
    Text(String),
    Number(u64),
}

/// Resolved options for Collator
#[derive(Debug, Clone)]
pub struct CollatorResolvedOptions {
    /// The locale tag
    pub locale: String,
    /// The usage type
    pub usage: CollatorUsage,
    /// The sensitivity level
    pub sensitivity: CollatorSensitivity,
    /// Whether punctuation is ignored
    pub ignore_punctuation: bool,
    /// Whether numeric collation is used
    pub numeric: bool,
    /// Case ordering
    pub case_first: CaseFirst,
}

// ============================================================================
// NumberFormat
// ============================================================================

/// Number formatting style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberStyle {
    /// Plain number formatting
    Decimal,
    /// Currency formatting
    Currency,
    /// Percentage formatting
    Percent,
    /// Unit formatting
    Unit,
}

impl Default for NumberStyle {
    fn default() -> Self {
        NumberStyle::Decimal
    }
}

/// Currency display mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrencyDisplay {
    /// Currency symbol (e.g., "$")
    Symbol,
    /// Narrow symbol (e.g., "$" instead of "US$")
    NarrowSymbol,
    /// Currency code (e.g., "USD")
    Code,
    /// Currency name (e.g., "US dollars")
    Name,
}

impl Default for CurrencyDisplay {
    fn default() -> Self {
        CurrencyDisplay::Symbol
    }
}

/// Sign display mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignDisplay {
    /// Sign for negative numbers only
    Auto,
    /// Always show sign
    Always,
    /// Never show sign
    Never,
    /// Show sign except for zero
    ExceptZero,
}

impl Default for SignDisplay {
    fn default() -> Self {
        SignDisplay::Auto
    }
}

/// Notation style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notation {
    /// Standard notation
    Standard,
    /// Scientific notation (e.g., 1.23E4)
    Scientific,
    /// Engineering notation (exponent multiple of 3)
    Engineering,
    /// Compact notation (e.g., "1.2K")
    Compact,
}

impl Default for Notation {
    fn default() -> Self {
        Notation::Standard
    }
}

/// Compact display mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactDisplay {
    /// Short form (e.g., "1.2K")
    Short,
    /// Long form (e.g., "1.2 thousand")
    Long,
}

impl Default for CompactDisplay {
    fn default() -> Self {
        CompactDisplay::Short
    }
}

/// Options for NumberFormat
#[derive(Debug, Clone)]
pub struct NumberFormatOptions {
    /// The formatting style
    pub style: NumberStyle,
    /// Currency code (required for currency style)
    pub currency: Option<String>,
    /// How to display the currency
    pub currency_display: CurrencyDisplay,
    /// How to display the sign
    pub sign_display: SignDisplay,
    /// Notation style
    pub notation: Notation,
    /// Compact display mode
    pub compact_display: CompactDisplay,
    /// Whether to use grouping separators
    pub use_grouping: bool,
    /// Minimum integer digits
    pub minimum_integer_digits: u8,
    /// Minimum fraction digits
    pub minimum_fraction_digits: Option<u8>,
    /// Maximum fraction digits
    pub maximum_fraction_digits: Option<u8>,
    /// Minimum significant digits
    pub minimum_significant_digits: Option<u8>,
    /// Maximum significant digits
    pub maximum_significant_digits: Option<u8>,
}

impl Default for NumberFormatOptions {
    fn default() -> Self {
        NumberFormatOptions {
            style: NumberStyle::Decimal,
            currency: None,
            currency_display: CurrencyDisplay::Symbol,
            sign_display: SignDisplay::Auto,
            notation: Notation::Standard,
            compact_display: CompactDisplay::Short,
            use_grouping: true,
            minimum_integer_digits: 1,
            minimum_fraction_digits: None,
            maximum_fraction_digits: None,
            minimum_significant_digits: None,
            maximum_significant_digits: None,
        }
    }
}

impl NumberFormatOptions {
    /// Create options for currency formatting
    pub fn currency(code: &str) -> Self {
        NumberFormatOptions {
            style: NumberStyle::Currency,
            currency: Some(code.to_uppercase()),
            minimum_fraction_digits: Some(2),
            maximum_fraction_digits: Some(2),
            ..Default::default()
        }
    }

    /// Create options for percentage formatting
    pub fn percent() -> Self {
        NumberFormatOptions {
            style: NumberStyle::Percent,
            minimum_fraction_digits: Some(0),
            maximum_fraction_digits: Some(0),
            ..Default::default()
        }
    }

    /// Create options for compact notation
    pub fn compact() -> Self {
        NumberFormatOptions {
            notation: Notation::Compact,
            ..Default::default()
        }
    }
}

/// Locale-sensitive number formatting
#[derive(Debug, Clone)]
pub struct NumberFormat {
    locale: Locale,
    options: NumberFormatOptions,
}

impl NumberFormat {
    /// Create a new NumberFormat with the given locale and options
    pub fn new(locale: Locale, options: NumberFormatOptions) -> Self {
        NumberFormat { locale, options }
    }

    /// Create a NumberFormat with default options
    pub fn with_locale(locale: Locale) -> Self {
        NumberFormat {
            locale,
            options: NumberFormatOptions::default(),
        }
    }

    /// Format a number according to the locale and options
    pub fn format(&self, value: f64) -> String {
        if value.is_nan() {
            return "NaN".to_string();
        }

        if value.is_infinite() {
            return if value > 0.0 {
                "∞".to_string()
            } else {
                "-∞".to_string()
            };
        }

        match self.options.notation {
            Notation::Scientific => return self.format_scientific(value),
            Notation::Engineering => return self.format_engineering(value),
            Notation::Compact => return self.format_compact(value),
            Notation::Standard => {}
        }

        match self.options.style {
            NumberStyle::Currency => self.format_currency(value),
            NumberStyle::Percent => self.format_percent(value),
            NumberStyle::Unit => self.format_decimal(value), // Simplified
            NumberStyle::Decimal => self.format_decimal(value),
        }
    }

    /// Format a number to parts
    pub fn format_to_parts(&self, value: f64) -> Vec<NumberFormatPart> {
        let mut parts = Vec::new();
        let formatted = self.format(value);

        // Simplified implementation - just returns the formatted string as a literal
        parts.push(NumberFormatPart {
            part_type: "literal".to_string(),
            value: formatted,
        });

        parts
    }

    /// Get the resolved options
    pub fn resolved_options(&self) -> NumberFormatResolvedOptions {
        NumberFormatResolvedOptions {
            locale: self.locale.to_string(),
            style: self.options.style,
            currency: self.options.currency.clone(),
            currency_display: self.options.currency_display,
            use_grouping: self.options.use_grouping,
            minimum_integer_digits: self.options.minimum_integer_digits,
            minimum_fraction_digits: self.get_min_fraction_digits(),
            maximum_fraction_digits: self.get_max_fraction_digits(),
        }
    }

    /// Format as decimal
    fn format_decimal(&self, value: f64) -> String {
        let sign = self.format_sign(value);
        let abs_value = value.abs();

        let min_frac = self.get_min_fraction_digits();
        let max_frac = self.get_max_fraction_digits();

        let formatted = self.format_number_with_digits(abs_value, min_frac, max_frac);
        let with_grouping = if self.options.use_grouping {
            self.add_grouping_separators(&formatted)
        } else {
            formatted
        };

        format!("{}{}", sign, with_grouping)
    }

    /// Format as currency
    fn format_currency(&self, value: f64) -> String {
        let sign = self.format_sign(value);
        let abs_value = value.abs();

        let min_frac = self.options.minimum_fraction_digits.unwrap_or(2);
        let max_frac = self.options.maximum_fraction_digits.unwrap_or(2);

        let formatted = self.format_number_with_digits(abs_value, min_frac, max_frac);
        let with_grouping = if self.options.use_grouping {
            self.add_grouping_separators(&formatted)
        } else {
            formatted
        };

        let currency_str = self.get_currency_string();

        // Format based on locale
        match self.locale.language.as_str() {
            "de" | "fr" | "es" | "it" | "pt" | "ru" => {
                // Currency after number with space
                format!("{}{} {}", sign, with_grouping, currency_str)
            }
            _ => {
                // Currency before number (US, UK, etc.)
                format!("{}{}{}", sign, currency_str, with_grouping)
            }
        }
    }

    /// Format as percentage
    fn format_percent(&self, value: f64) -> String {
        let sign = self.format_sign(value);
        let percent_value = value.abs() * 100.0;

        let min_frac = self.options.minimum_fraction_digits.unwrap_or(0);
        let max_frac = self.options.maximum_fraction_digits.unwrap_or(0);

        let formatted = self.format_number_with_digits(percent_value, min_frac, max_frac);

        format!("{}{}%", sign, formatted)
    }

    /// Format in scientific notation
    fn format_scientific(&self, value: f64) -> String {
        if value == 0.0 {
            return "0E0".to_string();
        }

        let sign = self.format_sign(value);
        let abs_value = value.abs();
        let exponent = abs_value.log10().floor() as i32;
        let mantissa = abs_value / 10f64.powi(exponent);

        let max_frac = self.get_max_fraction_digits();
        let mantissa_str = self.format_number_with_digits(mantissa, 0, max_frac);

        format!("{}{}E{}", sign, mantissa_str, exponent)
    }

    /// Format in engineering notation
    fn format_engineering(&self, value: f64) -> String {
        if value == 0.0 {
            return "0E0".to_string();
        }

        let sign = self.format_sign(value);
        let abs_value = value.abs();
        let exponent = abs_value.log10().floor() as i32;
        let eng_exponent = (exponent / 3) * 3;
        let mantissa = abs_value / 10f64.powi(eng_exponent);

        let max_frac = self.get_max_fraction_digits();
        let mantissa_str = self.format_number_with_digits(mantissa, 0, max_frac);

        format!("{}{}E{}", sign, mantissa_str, eng_exponent)
    }

    /// Format in compact notation
    fn format_compact(&self, value: f64) -> String {
        let sign = self.format_sign(value);
        let abs_value = value.abs();

        let (scaled, suffix) = if abs_value >= 1_000_000_000_000.0 {
            (abs_value / 1_000_000_000_000.0, "T")
        } else if abs_value >= 1_000_000_000.0 {
            (abs_value / 1_000_000_000.0, "B")
        } else if abs_value >= 1_000_000.0 {
            (abs_value / 1_000_000.0, "M")
        } else if abs_value >= 1_000.0 {
            (abs_value / 1_000.0, "K")
        } else {
            (abs_value, "")
        };

        let formatted = if scaled >= 100.0 {
            format!("{:.0}", scaled)
        } else if scaled >= 10.0 {
            format!("{:.1}", scaled)
        } else {
            format!("{:.2}", scaled)
        };

        // Trim trailing zeros and decimal point
        let trimmed = formatted
            .trim_end_matches('0')
            .trim_end_matches('.');

        format!("{}{}{}", sign, trimmed, suffix)
    }

    /// Format the sign
    fn format_sign(&self, value: f64) -> &'static str {
        match self.options.sign_display {
            SignDisplay::Auto => {
                if value < 0.0 {
                    "-"
                } else {
                    ""
                }
            }
            SignDisplay::Always => {
                if value < 0.0 {
                    "-"
                } else if value > 0.0 {
                    "+"
                } else {
                    ""
                }
            }
            SignDisplay::Never => "",
            SignDisplay::ExceptZero => {
                if value < 0.0 {
                    "-"
                } else if value > 0.0 {
                    "+"
                } else {
                    ""
                }
            }
        }
    }

    /// Format number with specified digit constraints
    fn format_number_with_digits(&self, value: f64, min_frac: u8, max_frac: u8) -> String {
        let formatted = format!("{:.prec$}", value, prec = max_frac as usize);

        // Split into integer and fraction parts
        let parts: Vec<&str> = formatted.split('.').collect();
        let mut integer_part = parts[0].to_string();
        let fraction_part = parts.get(1).map(|s| s.to_string());

        // Pad integer part
        let min_int = self.options.minimum_integer_digits as usize;
        while integer_part.len() < min_int {
            integer_part.insert(0, '0');
        }

        match fraction_part {
            Some(frac) if min_frac > 0 || !frac.chars().all(|c| c == '0') => {
                // Trim trailing zeros but keep minimum
                let mut trimmed: String = frac
                    .chars()
                    .rev()
                    .skip_while(|&c| c == '0')
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect();

                while trimmed.len() < min_frac as usize {
                    trimmed.push('0');
                }

                if trimmed.is_empty() && min_frac == 0 {
                    integer_part
                } else {
                    format!("{}.{}", integer_part, trimmed)
                }
            }
            _ if min_frac > 0 => {
                format!("{}.{}", integer_part, "0".repeat(min_frac as usize))
            }
            _ => integer_part,
        }
    }

    /// Add grouping separators (thousands separators)
    fn add_grouping_separators(&self, s: &str) -> String {
        let parts: Vec<&str> = s.split('.').collect();
        let integer_part = parts[0];
        let fraction_part = parts.get(1);

        let separator = self.get_grouping_separator();
        let decimal = self.get_decimal_separator();

        let mut result = String::new();
        let chars: Vec<char> = integer_part.chars().collect();
        let len = chars.len();

        for (i, c) in chars.iter().enumerate() {
            if i > 0 && (len - i) % 3 == 0 {
                result.push(separator);
            }
            result.push(*c);
        }

        match fraction_part {
            Some(frac) => format!("{}{}{}", result, decimal, frac),
            None => result,
        }
    }

    /// Get the grouping separator for the locale
    fn get_grouping_separator(&self) -> char {
        match self.locale.language.as_str() {
            "de" | "fr" | "es" | "it" | "pt" | "ru" | "pl" | "nl" => ' ',
            _ => ',',
        }
    }

    /// Get the decimal separator for the locale
    fn get_decimal_separator(&self) -> char {
        match self.locale.language.as_str() {
            "de" | "fr" | "es" | "it" | "pt" | "ru" | "pl" | "nl" => ',',
            _ => '.',
        }
    }

    /// Get the currency string
    fn get_currency_string(&self) -> String {
        let code = self.options.currency.as_deref().unwrap_or("USD");

        match self.options.currency_display {
            CurrencyDisplay::Code => code.to_string(),
            CurrencyDisplay::Name => self.get_currency_name(code),
            CurrencyDisplay::Symbol | CurrencyDisplay::NarrowSymbol => {
                self.get_currency_symbol(code)
            }
        }
    }

    /// Get currency symbol
    fn get_currency_symbol(&self, code: &str) -> String {
        match code {
            "USD" => "$".to_string(),
            "EUR" => "€".to_string(),
            "GBP" => "£".to_string(),
            "JPY" => "¥".to_string(),
            "CNY" => "¥".to_string(),
            "KRW" => "₩".to_string(),
            "INR" => "₹".to_string(),
            "RUB" => "₽".to_string(),
            "BRL" => "R$".to_string(),
            "CHF" => "CHF".to_string(),
            "CAD" => "CA$".to_string(),
            "AUD" => "A$".to_string(),
            "MXN" => "MX$".to_string(),
            _ => code.to_string(),
        }
    }

    /// Get currency name
    fn get_currency_name(&self, code: &str) -> String {
        match code {
            "USD" => "US dollars".to_string(),
            "EUR" => "euros".to_string(),
            "GBP" => "British pounds".to_string(),
            "JPY" => "Japanese yen".to_string(),
            "CNY" => "Chinese yuan".to_string(),
            _ => code.to_string(),
        }
    }

    fn get_min_fraction_digits(&self) -> u8 {
        self.options.minimum_fraction_digits.unwrap_or(0)
    }

    fn get_max_fraction_digits(&self) -> u8 {
        self.options.maximum_fraction_digits.unwrap_or(3)
    }
}

/// A part of a formatted number
#[derive(Debug, Clone)]
pub struct NumberFormatPart {
    /// The type of this part
    pub part_type: String,
    /// The value of this part
    pub value: String,
}

/// Resolved options for NumberFormat
#[derive(Debug, Clone)]
pub struct NumberFormatResolvedOptions {
    /// The locale tag
    pub locale: String,
    /// The formatting style
    pub style: NumberStyle,
    /// The currency code
    pub currency: Option<String>,
    /// How the currency is displayed
    pub currency_display: CurrencyDisplay,
    /// Whether grouping is used
    pub use_grouping: bool,
    /// Minimum integer digits
    pub minimum_integer_digits: u8,
    /// Minimum fraction digits
    pub minimum_fraction_digits: u8,
    /// Maximum fraction digits
    pub maximum_fraction_digits: u8,
}

// ============================================================================
// DateTimeFormat
// ============================================================================

/// Date/time formatting style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateTimeStyle {
    /// Full format (e.g., "Tuesday, April 12, 2044")
    Full,
    /// Long format (e.g., "April 12, 2044")
    Long,
    /// Medium format (e.g., "Apr 12, 2044")
    Medium,
    /// Short format (e.g., "4/12/44")
    Short,
}

/// Hour cycle options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HourCycle {
    /// 12-hour clock, midnight at 12
    H12,
    /// 24-hour clock, midnight at 0
    H23,
    /// 12-hour clock, midnight at 0
    H11,
    /// 24-hour clock, midnight at 24
    H24,
}

/// Options for DateTimeFormat
#[derive(Debug, Clone, Default)]
pub struct DateTimeFormatOptions {
    /// Date style
    pub date_style: Option<DateTimeStyle>,
    /// Time style
    pub time_style: Option<DateTimeStyle>,
    /// Hour cycle
    pub hour_cycle: Option<HourCycle>,
    /// Time zone
    pub time_zone: Option<String>,
    /// Era (e.g., "AD", "BC")
    pub era: Option<String>,
    /// Year format ("numeric", "2-digit")
    pub year: Option<String>,
    /// Month format ("numeric", "2-digit", "long", "short", "narrow")
    pub month: Option<String>,
    /// Day format ("numeric", "2-digit")
    pub day: Option<String>,
    /// Weekday format ("long", "short", "narrow")
    pub weekday: Option<String>,
    /// Hour format ("numeric", "2-digit")
    pub hour: Option<String>,
    /// Minute format ("numeric", "2-digit")
    pub minute: Option<String>,
    /// Second format ("numeric", "2-digit")
    pub second: Option<String>,
    /// Time zone name format ("long", "short")
    pub time_zone_name: Option<String>,
}

impl DateTimeFormatOptions {
    /// Create options for date-only formatting
    pub fn date_only(style: DateTimeStyle) -> Self {
        DateTimeFormatOptions {
            date_style: Some(style),
            ..Default::default()
        }
    }

    /// Create options for time-only formatting
    pub fn time_only(style: DateTimeStyle) -> Self {
        DateTimeFormatOptions {
            time_style: Some(style),
            ..Default::default()
        }
    }

    /// Create options for date and time formatting
    pub fn date_time(date_style: DateTimeStyle, time_style: DateTimeStyle) -> Self {
        DateTimeFormatOptions {
            date_style: Some(date_style),
            time_style: Some(time_style),
            ..Default::default()
        }
    }
}

/// Locale-sensitive date/time formatting
#[derive(Debug, Clone)]
pub struct DateTimeFormat {
    locale: Locale,
    options: DateTimeFormatOptions,
}

impl DateTimeFormat {
    /// Create a new DateTimeFormat with the given locale and options
    pub fn new(locale: Locale, options: DateTimeFormatOptions) -> Self {
        DateTimeFormat { locale, options }
    }

    /// Create a DateTimeFormat with default options
    pub fn with_locale(locale: Locale) -> Self {
        DateTimeFormat {
            locale,
            options: DateTimeFormatOptions::default(),
        }
    }

    /// Format a timestamp (milliseconds since Unix epoch)
    pub fn format(&self, timestamp_ms: f64) -> String {
        if timestamp_ms.is_nan() || timestamp_ms.is_infinite() {
            return "Invalid Date".to_string();
        }

        // Convert to components
        let secs = (timestamp_ms / 1000.0).floor() as i64;
        let (year, month, day, hour, minute, second, weekday) = Self::timestamp_to_components(secs);

        let date_str = self.format_date_part(year, month, day, weekday);
        let time_str = self.format_time_part(hour, minute, second);

        match (&self.options.date_style, &self.options.time_style) {
            (Some(_), Some(_)) => format!("{}, {}", date_str, time_str),
            (Some(_), None) => date_str,
            (None, Some(_)) => time_str,
            (None, None) => {
                // Default: show both
                format!("{}, {}", date_str, time_str)
            }
        }
    }

    /// Format a Date object
    pub fn format_date(&self, date: &crate::date::JsDate) -> String {
        self.format(date.get_time())
    }

    /// Get the resolved options
    pub fn resolved_options(&self) -> DateTimeFormatResolvedOptions {
        DateTimeFormatResolvedOptions {
            locale: self.locale.to_string(),
            date_style: self.options.date_style,
            time_style: self.options.time_style,
            time_zone: self.options.time_zone.clone().unwrap_or_else(|| "UTC".to_string()),
            hour_cycle: self.options.hour_cycle,
        }
    }

    /// Convert timestamp to date components
    fn timestamp_to_components(secs: i64) -> (i32, u32, u32, u32, u32, u32, u32) {
        // Simple implementation - doesn't handle all edge cases
        let days_since_epoch = secs / 86400;
        let time_of_day = secs % 86400;

        let hour = ((time_of_day / 3600) % 24) as u32;
        let minute = ((time_of_day % 3600) / 60) as u32;
        let second = (time_of_day % 60) as u32;

        // Calculate weekday (0 = Sunday)
        // Jan 1, 1970 was a Thursday (4)
        let weekday = ((days_since_epoch + 4) % 7) as u32;

        // Calculate year, month, day using a simplified algorithm
        let (year, month, day) = Self::days_to_ymd(days_since_epoch as i32);

        (year, month, day, hour, minute, second, weekday)
    }

    /// Convert days since epoch to year, month, day
    fn days_to_ymd(days: i32) -> (i32, u32, u32) {
        // Simplified implementation
        let mut remaining = days;
        let mut year = 1970i32;

        // Handle negative days
        while remaining < 0 {
            year -= 1;
            let days_in_year = if Self::is_leap_year(year) { 366 } else { 365 };
            remaining += days_in_year;
        }

        // Count forward through years
        loop {
            let days_in_year = if Self::is_leap_year(year) { 366 } else { 365 };
            if remaining < days_in_year {
                break;
            }
            remaining -= days_in_year;
            year += 1;
        }

        // Count through months
        let is_leap = Self::is_leap_year(year);
        let days_in_months = [
            31,
            if is_leap { 29 } else { 28 },
            31,
            30,
            31,
            30,
            31,
            31,
            30,
            31,
            30,
            31,
        ];

        let mut month = 0u32;
        for (i, &days_in_month) in days_in_months.iter().enumerate() {
            if remaining < days_in_month {
                month = (i + 1) as u32;
                break;
            }
            remaining -= days_in_month;
        }

        let day = (remaining + 1) as u32;

        (year, month, day)
    }

    /// Check if a year is a leap year
    fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    /// Format the date part
    fn format_date_part(&self, year: i32, month: u32, day: u32, weekday: u32) -> String {
        let style = self.options.date_style.unwrap_or(DateTimeStyle::Medium);

        let month_name = self.get_month_name(month, style);
        let weekday_name = self.get_weekday_name(weekday, style);

        match style {
            DateTimeStyle::Full => {
                format!("{}, {} {}, {}", weekday_name, month_name, day, year)
            }
            DateTimeStyle::Long => {
                format!("{} {}, {}", month_name, day, year)
            }
            DateTimeStyle::Medium => {
                format!("{} {}, {}", month_name, day, year)
            }
            DateTimeStyle::Short => {
                // Use locale-appropriate order
                match self.locale.language.as_str() {
                    "en" => format!("{}/{}/{}", month, day, year % 100),
                    "de" | "fr" | "es" | "it" | "ru" => format!("{}/{}/{}", day, month, year % 100),
                    "ja" | "zh" | "ko" => format!("{}/{}/{}", year % 100, month, day),
                    _ => format!("{}/{}/{}", month, day, year % 100),
                }
            }
        }
    }

    /// Format the time part
    fn format_time_part(&self, hour: u32, minute: u32, second: u32) -> String {
        let style = self.options.time_style.unwrap_or(DateTimeStyle::Medium);
        let use_12_hour = matches!(
            self.options.hour_cycle,
            Some(HourCycle::H12) | Some(HourCycle::H11)
        ) || (self.options.hour_cycle.is_none()
            && matches!(self.locale.language.as_str(), "en"));

        let (display_hour, period) = if use_12_hour {
            let h = if hour == 0 {
                12
            } else if hour > 12 {
                hour - 12
            } else {
                hour
            };
            let p = if hour < 12 { "AM" } else { "PM" };
            (h, Some(p))
        } else {
            (hour, None)
        };

        let time_str = match style {
            DateTimeStyle::Full | DateTimeStyle::Long => {
                format!("{:02}:{:02}:{:02}", display_hour, minute, second)
            }
            DateTimeStyle::Medium => {
                format!("{:02}:{:02}:{:02}", display_hour, minute, second)
            }
            DateTimeStyle::Short => {
                format!("{:02}:{:02}", display_hour, minute)
            }
        };

        match period {
            Some(p) => format!("{} {}", time_str, p),
            None => time_str,
        }
    }

    /// Get month name
    fn get_month_name(&self, month: u32, style: DateTimeStyle) -> String {
        let names_long = [
            "January", "February", "March", "April", "May", "June",
            "July", "August", "September", "October", "November", "December",
        ];
        let names_short = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun",
            "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];

        let idx = (month.saturating_sub(1)) as usize;
        if idx >= 12 {
            return month.to_string();
        }

        match style {
            DateTimeStyle::Full | DateTimeStyle::Long => names_long[idx].to_string(),
            DateTimeStyle::Medium => names_short[idx].to_string(),
            DateTimeStyle::Short => month.to_string(),
        }
    }

    /// Get weekday name
    fn get_weekday_name(&self, weekday: u32, style: DateTimeStyle) -> String {
        let names_long = [
            "Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday",
        ];
        let names_short = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

        let idx = (weekday % 7) as usize;

        match style {
            DateTimeStyle::Full => names_long[idx].to_string(),
            DateTimeStyle::Long | DateTimeStyle::Medium => names_short[idx].to_string(),
            DateTimeStyle::Short => "".to_string(),
        }
    }
}

/// Resolved options for DateTimeFormat
#[derive(Debug, Clone)]
pub struct DateTimeFormatResolvedOptions {
    /// The locale tag
    pub locale: String,
    /// The date style
    pub date_style: Option<DateTimeStyle>,
    /// The time style
    pub time_style: Option<DateTimeStyle>,
    /// The time zone
    pub time_zone: String,
    /// The hour cycle
    pub hour_cycle: Option<HourCycle>,
}

// ============================================================================
// PluralRules
// ============================================================================

/// Plural categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluralCategory {
    /// Zero quantity
    Zero,
    /// One item
    One,
    /// Two items
    Two,
    /// Few items (language-specific)
    Few,
    /// Many items (language-specific)
    Many,
    /// Other (default)
    Other,
}

impl PluralCategory {
    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            PluralCategory::Zero => "zero",
            PluralCategory::One => "one",
            PluralCategory::Two => "two",
            PluralCategory::Few => "few",
            PluralCategory::Many => "many",
            PluralCategory::Other => "other",
        }
    }
}

/// Plural rules type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluralRulesType {
    /// Cardinal numbers (e.g., "1 dog", "2 dogs")
    Cardinal,
    /// Ordinal numbers (e.g., "1st", "2nd", "3rd")
    Ordinal,
}

impl Default for PluralRulesType {
    fn default() -> Self {
        PluralRulesType::Cardinal
    }
}

/// Options for PluralRules
#[derive(Debug, Clone, Default)]
pub struct PluralRulesOptions {
    /// The type of plural rules
    pub plural_type: PluralRulesType,
    /// Minimum integer digits
    pub minimum_integer_digits: Option<u8>,
    /// Minimum fraction digits
    pub minimum_fraction_digits: Option<u8>,
    /// Maximum fraction digits
    pub maximum_fraction_digits: Option<u8>,
    /// Minimum significant digits
    pub minimum_significant_digits: Option<u8>,
    /// Maximum significant digits
    pub maximum_significant_digits: Option<u8>,
}

/// Plural-sensitive formatting
#[derive(Debug, Clone)]
pub struct PluralRules {
    locale: Locale,
    options: PluralRulesOptions,
}

impl PluralRules {
    /// Create new PluralRules with the given locale and options
    pub fn new(locale: Locale, options: PluralRulesOptions) -> Self {
        PluralRules { locale, options }
    }

    /// Create PluralRules with default options
    pub fn with_locale(locale: Locale) -> Self {
        PluralRules {
            locale,
            options: PluralRulesOptions::default(),
        }
    }

    /// Select the plural category for a number
    pub fn select(&self, n: f64) -> PluralCategory {
        match self.options.plural_type {
            PluralRulesType::Cardinal => self.select_cardinal(n),
            PluralRulesType::Ordinal => self.select_ordinal(n),
        }
    }

    /// Select plural category for cardinal numbers
    fn select_cardinal(&self, n: f64) -> PluralCategory {
        let abs_n = n.abs();
        let i = abs_n.trunc() as i64;
        let has_decimal = abs_n != abs_n.trunc();

        match self.locale.language.as_str() {
            // English: 1 = one, else other
            "en" => {
                if i == 1 && !has_decimal {
                    PluralCategory::One
                } else {
                    PluralCategory::Other
                }
            }
            // French: 0-1 = one, else other
            "fr" => {
                if abs_n >= 0.0 && abs_n < 2.0 {
                    PluralCategory::One
                } else {
                    PluralCategory::Other
                }
            }
            // Russian: complex rules
            "ru" | "uk" | "pl" => self.select_slavic_cardinal(i, has_decimal),
            // Arabic: 0 = zero, 1 = one, 2 = two, 3-10 = few, 11-99 = many
            "ar" => {
                if i == 0 && !has_decimal {
                    PluralCategory::Zero
                } else if i == 1 && !has_decimal {
                    PluralCategory::One
                } else if i == 2 && !has_decimal {
                    PluralCategory::Two
                } else if i >= 3 && i <= 10 && !has_decimal {
                    PluralCategory::Few
                } else if i >= 11 && i <= 99 && !has_decimal {
                    PluralCategory::Many
                } else {
                    PluralCategory::Other
                }
            }
            // Japanese, Chinese, Korean: no plural distinction
            "ja" | "zh" | "ko" => PluralCategory::Other,
            // Default: simple one/other
            _ => {
                if i == 1 && !has_decimal {
                    PluralCategory::One
                } else {
                    PluralCategory::Other
                }
            }
        }
    }

    /// Select plural category for Slavic languages (Russian, Ukrainian, Polish)
    fn select_slavic_cardinal(&self, i: i64, has_decimal: bool) -> PluralCategory {
        if has_decimal {
            return PluralCategory::Other;
        }

        let mod10 = i % 10;
        let mod100 = i % 100;

        if mod10 == 1 && mod100 != 11 {
            PluralCategory::One
        } else if mod10 >= 2 && mod10 <= 4 && !(mod100 >= 12 && mod100 <= 14) {
            PluralCategory::Few
        } else if mod10 == 0 || (mod10 >= 5 && mod10 <= 9) || (mod100 >= 11 && mod100 <= 14) {
            PluralCategory::Many
        } else {
            PluralCategory::Other
        }
    }

    /// Select plural category for ordinal numbers
    fn select_ordinal(&self, n: f64) -> PluralCategory {
        let i = n.abs().trunc() as i64;

        match self.locale.language.as_str() {
            // English ordinals: 1st, 2nd, 3rd, Nth
            "en" => {
                let mod10 = i % 10;
                let mod100 = i % 100;

                if mod10 == 1 && mod100 != 11 {
                    PluralCategory::One // 1st, 21st, 31st
                } else if mod10 == 2 && mod100 != 12 {
                    PluralCategory::Two // 2nd, 22nd, 32nd
                } else if mod10 == 3 && mod100 != 13 {
                    PluralCategory::Few // 3rd, 23rd, 33rd
                } else {
                    PluralCategory::Other // 4th, 5th, 11th, 12th, 13th
                }
            }
            // Most languages don't have special ordinal forms
            _ => PluralCategory::Other,
        }
    }

    /// Get the resolved options
    pub fn resolved_options(&self) -> PluralRulesResolvedOptions {
        PluralRulesResolvedOptions {
            locale: self.locale.to_string(),
            plural_type: self.options.plural_type,
            minimum_integer_digits: self.options.minimum_integer_digits.unwrap_or(1),
            minimum_fraction_digits: self.options.minimum_fraction_digits.unwrap_or(0),
            maximum_fraction_digits: self.options.maximum_fraction_digits.unwrap_or(3),
        }
    }
}

/// Resolved options for PluralRules
#[derive(Debug, Clone)]
pub struct PluralRulesResolvedOptions {
    /// The locale tag
    pub locale: String,
    /// The type of plural rules
    pub plural_type: PluralRulesType,
    /// Minimum integer digits
    pub minimum_integer_digits: u8,
    /// Minimum fraction digits
    pub minimum_fraction_digits: u8,
    /// Maximum fraction digits
    pub maximum_fraction_digits: u8,
}

// ============================================================================
// RelativeTimeFormat
// ============================================================================

/// Relative time unit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelativeTimeUnit {
    /// Years
    Year,
    /// Quarters
    Quarter,
    /// Months
    Month,
    /// Weeks
    Week,
    /// Days
    Day,
    /// Hours
    Hour,
    /// Minutes
    Minute,
    /// Seconds
    Second,
}

impl RelativeTimeUnit {
    /// Convert from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "year" | "years" => Some(RelativeTimeUnit::Year),
            "quarter" | "quarters" => Some(RelativeTimeUnit::Quarter),
            "month" | "months" => Some(RelativeTimeUnit::Month),
            "week" | "weeks" => Some(RelativeTimeUnit::Week),
            "day" | "days" => Some(RelativeTimeUnit::Day),
            "hour" | "hours" => Some(RelativeTimeUnit::Hour),
            "minute" | "minutes" => Some(RelativeTimeUnit::Minute),
            "second" | "seconds" => Some(RelativeTimeUnit::Second),
            _ => None,
        }
    }
}

/// Relative time format style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelativeTimeStyle {
    /// Long format (e.g., "in 1 day")
    Long,
    /// Short format (e.g., "in 1 day")
    Short,
    /// Narrow format (e.g., "in 1d")
    Narrow,
}

impl Default for RelativeTimeStyle {
    fn default() -> Self {
        RelativeTimeStyle::Long
    }
}

/// Numeric option for relative time
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelativeTimeNumeric {
    /// Always use numeric format (e.g., "1 day ago")
    Always,
    /// Use phrases when available (e.g., "yesterday")
    Auto,
}

impl Default for RelativeTimeNumeric {
    fn default() -> Self {
        RelativeTimeNumeric::Always
    }
}

/// Options for RelativeTimeFormat
#[derive(Debug, Clone, Default)]
pub struct RelativeTimeFormatOptions {
    /// The formatting style
    pub style: RelativeTimeStyle,
    /// Whether to use numeric format
    pub numeric: RelativeTimeNumeric,
}

/// Relative time formatting
#[derive(Debug, Clone)]
pub struct RelativeTimeFormat {
    locale: Locale,
    options: RelativeTimeFormatOptions,
}

impl RelativeTimeFormat {
    /// Create a new RelativeTimeFormat with the given locale and options
    pub fn new(locale: Locale, options: RelativeTimeFormatOptions) -> Self {
        RelativeTimeFormat { locale, options }
    }

    /// Create a RelativeTimeFormat with default options
    pub fn with_locale(locale: Locale) -> Self {
        RelativeTimeFormat {
            locale,
            options: RelativeTimeFormatOptions::default(),
        }
    }

    /// Format a relative time value
    pub fn format(&self, value: f64, unit: RelativeTimeUnit) -> String {
        let abs_value = value.abs();
        let is_future = value > 0.0;
        let integer_value = abs_value.round() as i64;

        // Check for special cases when using auto numeric
        if matches!(self.options.numeric, RelativeTimeNumeric::Auto) {
            if let Some(special) = self.get_special_phrase(integer_value, unit, is_future) {
                return special;
            }
        }

        let unit_str = self.get_unit_string(unit, abs_value);

        if is_future {
            format!("in {} {}", self.format_number(abs_value), unit_str)
        } else {
            format!("{} {} ago", self.format_number(abs_value), unit_str)
        }
    }

    /// Format a number for display
    fn format_number(&self, value: f64) -> String {
        if value == value.round() {
            format!("{}", value as i64)
        } else {
            format!("{:.1}", value)
        }
    }

    /// Get special phrase for common relative times
    fn get_special_phrase(&self, value: i64, unit: RelativeTimeUnit, is_future: bool) -> Option<String> {
        if value != 1 {
            return None;
        }

        match (unit, is_future) {
            (RelativeTimeUnit::Day, false) => Some("yesterday".to_string()),
            (RelativeTimeUnit::Day, true) => Some("tomorrow".to_string()),
            (RelativeTimeUnit::Week, false) => Some("last week".to_string()),
            (RelativeTimeUnit::Week, true) => Some("next week".to_string()),
            (RelativeTimeUnit::Month, false) => Some("last month".to_string()),
            (RelativeTimeUnit::Month, true) => Some("next month".to_string()),
            (RelativeTimeUnit::Year, false) => Some("last year".to_string()),
            (RelativeTimeUnit::Year, true) => Some("next year".to_string()),
            _ => None,
        }
    }

    /// Get the unit string
    fn get_unit_string(&self, unit: RelativeTimeUnit, value: f64) -> String {
        let plural = value != 1.0;

        match self.options.style {
            RelativeTimeStyle::Narrow => match unit {
                RelativeTimeUnit::Year => "yr".to_string(),
                RelativeTimeUnit::Quarter => "qtr".to_string(),
                RelativeTimeUnit::Month => "mo".to_string(),
                RelativeTimeUnit::Week => "wk".to_string(),
                RelativeTimeUnit::Day => "d".to_string(),
                RelativeTimeUnit::Hour => "hr".to_string(),
                RelativeTimeUnit::Minute => "min".to_string(),
                RelativeTimeUnit::Second => "sec".to_string(),
            },
            RelativeTimeStyle::Short => match unit {
                RelativeTimeUnit::Year => if plural { "yrs" } else { "yr" }.to_string(),
                RelativeTimeUnit::Quarter => if plural { "qtrs" } else { "qtr" }.to_string(),
                RelativeTimeUnit::Month => if plural { "mos" } else { "mo" }.to_string(),
                RelativeTimeUnit::Week => if plural { "wks" } else { "wk" }.to_string(),
                RelativeTimeUnit::Day => if plural { "days" } else { "day" }.to_string(),
                RelativeTimeUnit::Hour => if plural { "hrs" } else { "hr" }.to_string(),
                RelativeTimeUnit::Minute => if plural { "mins" } else { "min" }.to_string(),
                RelativeTimeUnit::Second => if plural { "secs" } else { "sec" }.to_string(),
            },
            RelativeTimeStyle::Long => match unit {
                RelativeTimeUnit::Year => if plural { "years" } else { "year" }.to_string(),
                RelativeTimeUnit::Quarter => if plural { "quarters" } else { "quarter" }.to_string(),
                RelativeTimeUnit::Month => if plural { "months" } else { "month" }.to_string(),
                RelativeTimeUnit::Week => if plural { "weeks" } else { "week" }.to_string(),
                RelativeTimeUnit::Day => if plural { "days" } else { "day" }.to_string(),
                RelativeTimeUnit::Hour => if plural { "hours" } else { "hour" }.to_string(),
                RelativeTimeUnit::Minute => if plural { "minutes" } else { "minute" }.to_string(),
                RelativeTimeUnit::Second => if plural { "seconds" } else { "second" }.to_string(),
            },
        }
    }

    /// Get the resolved options
    pub fn resolved_options(&self) -> RelativeTimeFormatResolvedOptions {
        RelativeTimeFormatResolvedOptions {
            locale: self.locale.to_string(),
            style: self.options.style,
            numeric: self.options.numeric,
        }
    }
}

/// Resolved options for RelativeTimeFormat
#[derive(Debug, Clone)]
pub struct RelativeTimeFormatResolvedOptions {
    /// The locale tag
    pub locale: String,
    /// The formatting style
    pub style: RelativeTimeStyle,
    /// The numeric option
    pub numeric: RelativeTimeNumeric,
}

// ============================================================================
// ListFormat
// ============================================================================

/// List format type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListFormatType {
    /// Conjunction (e.g., "A, B, and C")
    Conjunction,
    /// Disjunction (e.g., "A, B, or C")
    Disjunction,
    /// Unit (e.g., "A, B, C")
    Unit,
}

impl Default for ListFormatType {
    fn default() -> Self {
        ListFormatType::Conjunction
    }
}

/// List format style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListFormatStyle {
    /// Long style (e.g., "A, B, and C")
    Long,
    /// Short style (e.g., "A, B, & C")
    Short,
    /// Narrow style (e.g., "A, B, C")
    Narrow,
}

impl Default for ListFormatStyle {
    fn default() -> Self {
        ListFormatStyle::Long
    }
}

/// Options for ListFormat
#[derive(Debug, Clone, Default)]
pub struct ListFormatOptions {
    /// The type of list formatting
    pub list_type: ListFormatType,
    /// The style of list formatting
    pub style: ListFormatStyle,
}

/// Locale-sensitive list formatting
#[derive(Debug, Clone)]
pub struct ListFormat {
    locale: Locale,
    options: ListFormatOptions,
}

impl ListFormat {
    /// Create a new ListFormat with the given locale and options
    pub fn new(locale: Locale, options: ListFormatOptions) -> Self {
        ListFormat { locale, options }
    }

    /// Create a ListFormat with default options
    pub fn with_locale(locale: Locale) -> Self {
        ListFormat {
            locale,
            options: ListFormatOptions::default(),
        }
    }

    /// Format a list of strings
    pub fn format(&self, list: &[String]) -> String {
        match list.len() {
            0 => String::new(),
            1 => list[0].clone(),
            2 => self.format_two(&list[0], &list[1]),
            _ => self.format_many(list),
        }
    }

    /// Format a list of string slices
    pub fn format_str(&self, list: &[&str]) -> String {
        let owned: Vec<String> = list.iter().map(|s| s.to_string()).collect();
        self.format(&owned)
    }

    /// Format two items
    fn format_two(&self, a: &str, b: &str) -> String {
        let connector = self.get_pair_connector();
        format!("{}{}{}", a, connector, b)
    }

    /// Format more than two items
    fn format_many(&self, list: &[String]) -> String {
        let mut result = String::new();
        let len = list.len();

        for (i, item) in list.iter().enumerate() {
            if i == 0 {
                result.push_str(item);
            } else if i == len - 1 {
                let final_connector = self.get_final_connector();
                result.push_str(&final_connector);
                result.push_str(item);
            } else {
                let middle_connector = self.get_middle_connector();
                result.push_str(&middle_connector);
                result.push_str(item);
            }
        }

        result
    }

    /// Get the connector between two items
    fn get_pair_connector(&self) -> String {
        match self.options.list_type {
            ListFormatType::Conjunction => match self.options.style {
                ListFormatStyle::Long => " and ".to_string(),
                ListFormatStyle::Short => " & ".to_string(),
                ListFormatStyle::Narrow => ", ".to_string(),
            },
            ListFormatType::Disjunction => match self.options.style {
                ListFormatStyle::Long => " or ".to_string(),
                ListFormatStyle::Short => " or ".to_string(),
                ListFormatStyle::Narrow => ", ".to_string(),
            },
            ListFormatType::Unit => ", ".to_string(),
        }
    }

    /// Get the middle connector
    fn get_middle_connector(&self) -> String {
        ", ".to_string()
    }

    /// Get the final connector
    fn get_final_connector(&self) -> String {
        match self.options.list_type {
            ListFormatType::Conjunction => match self.options.style {
                ListFormatStyle::Long => {
                    // Check for Oxford comma based on locale
                    match self.locale.language.as_str() {
                        "en" => {
                            match self.locale.region.as_deref() {
                                Some("GB") | Some("AU") | Some("NZ") => " and ".to_string(),
                                _ => ", and ".to_string(), // US uses Oxford comma
                            }
                        }
                        _ => " and ".to_string(),
                    }
                }
                ListFormatStyle::Short => ", & ".to_string(),
                ListFormatStyle::Narrow => ", ".to_string(),
            },
            ListFormatType::Disjunction => match self.options.style {
                ListFormatStyle::Long => {
                    match self.locale.language.as_str() {
                        "en" => {
                            match self.locale.region.as_deref() {
                                Some("GB") | Some("AU") | Some("NZ") => " or ".to_string(),
                                _ => ", or ".to_string(),
                            }
                        }
                        _ => " or ".to_string(),
                    }
                }
                ListFormatStyle::Short => ", or ".to_string(),
                ListFormatStyle::Narrow => ", ".to_string(),
            },
            ListFormatType::Unit => ", ".to_string(),
        }
    }

    /// Format to parts
    pub fn format_to_parts(&self, list: &[String]) -> Vec<ListFormatPart> {
        let mut parts = Vec::new();

        for (i, item) in list.iter().enumerate() {
            if i > 0 {
                let connector = if i == list.len() - 1 {
                    self.get_final_connector()
                } else {
                    self.get_middle_connector()
                };
                parts.push(ListFormatPart {
                    part_type: "literal".to_string(),
                    value: connector,
                });
            }
            parts.push(ListFormatPart {
                part_type: "element".to_string(),
                value: item.clone(),
            });
        }

        parts
    }

    /// Get the resolved options
    pub fn resolved_options(&self) -> ListFormatResolvedOptions {
        ListFormatResolvedOptions {
            locale: self.locale.to_string(),
            list_type: self.options.list_type,
            style: self.options.style,
        }
    }
}

/// A part of a formatted list
#[derive(Debug, Clone)]
pub struct ListFormatPart {
    /// The type of this part
    pub part_type: String,
    /// The value of this part
    pub value: String,
}

/// Resolved options for ListFormat
#[derive(Debug, Clone)]
pub struct ListFormatResolvedOptions {
    /// The locale tag
    pub locale: String,
    /// The list type
    pub list_type: ListFormatType,
    /// The style
    pub style: ListFormatStyle,
}

// ============================================================================
// Intl Namespace
// ============================================================================

/// The Intl namespace object
pub struct Intl;

impl Intl {
    /// Get the canonical locale name
    pub fn get_canonical_locales(locales: &[&str]) -> JsResult<Vec<String>> {
        let mut result = Vec::new();

        for locale_str in locales {
            let locale = Locale::new(locale_str)?;
            result.push(locale.to_string());
        }

        Ok(result)
    }

    /// Get supported locales for a given constructor
    pub fn supported_locales_of(_available: &[&str], requested: &[&str]) -> Vec<String> {
        // Simplified implementation - returns all requested locales
        requested.iter().map(|s| s.to_string()).collect()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Locale Tests ====================

    #[test]
    fn test_locale_parse_simple() {
        let locale = Locale::new("en").unwrap();
        assert_eq!(locale.language, "en");
        assert_eq!(locale.script, None);
        assert_eq!(locale.region, None);
    }

    #[test]
    fn test_locale_parse_with_region() {
        let locale = Locale::new("en-US").unwrap();
        assert_eq!(locale.language, "en");
        assert_eq!(locale.region, Some("US".to_string()));
    }

    #[test]
    fn test_locale_parse_with_script() {
        let locale = Locale::new("zh-Hans-CN").unwrap();
        assert_eq!(locale.language, "zh");
        assert_eq!(locale.script, Some("Hans".to_string()));
        assert_eq!(locale.region, Some("CN".to_string()));
    }

    #[test]
    fn test_locale_to_string() {
        let locale = Locale::new("en-US").unwrap();
        assert_eq!(locale.to_string(), "en-US");
    }

    #[test]
    fn test_locale_maximize() {
        let locale = Locale::new("en").unwrap();
        let maximized = locale.maximize();
        assert_eq!(maximized.region, Some("US".to_string()));
    }

    #[test]
    fn test_locale_invalid() {
        assert!(Locale::new("").is_err());
        assert!(Locale::new("x").is_err()); // Too short
    }

    // ==================== Collator Tests ====================

    #[test]
    fn test_collator_basic_compare() {
        let collator = Collator::with_locale(Locale::default());

        assert_eq!(collator.compare("a", "b"), Ordering::Less);
        assert_eq!(collator.compare("b", "a"), Ordering::Greater);
        assert_eq!(collator.compare("a", "a"), Ordering::Equal);
    }

    #[test]
    fn test_collator_case_insensitive() {
        let options = CollatorOptions {
            sensitivity: CollatorSensitivity::Base,
            ..Default::default()
        };
        let collator = Collator::new(Locale::default(), options);

        assert_eq!(collator.compare("A", "a"), Ordering::Equal);
        assert_eq!(collator.compare("Apple", "apple"), Ordering::Equal);
    }

    #[test]
    fn test_collator_numeric() {
        let options = CollatorOptions {
            numeric: true,
            ..Default::default()
        };
        let collator = Collator::new(Locale::default(), options);

        // With numeric collation, "2" < "10"
        assert_eq!(collator.compare("file2.txt", "file10.txt"), Ordering::Less);
    }

    #[test]
    fn test_collator_ignore_punctuation() {
        let options = CollatorOptions {
            ignore_punctuation: true,
            ..Default::default()
        };
        let collator = Collator::new(Locale::default(), options);

        assert_eq!(collator.compare("hello", "hel.lo"), Ordering::Equal);
    }

    // ==================== NumberFormat Tests ====================

    #[test]
    fn test_number_format_decimal() {
        let formatter = NumberFormat::with_locale(Locale::new("en-US").unwrap());

        assert_eq!(formatter.format(1234.56), "1,234.56");
    }

    #[test]
    fn test_number_format_no_grouping() {
        let options = NumberFormatOptions {
            use_grouping: false,
            ..Default::default()
        };
        let formatter = NumberFormat::new(Locale::new("en-US").unwrap(), options);

        assert_eq!(formatter.format(1234567.0), "1234567");
    }

    #[test]
    fn test_number_format_currency() {
        let options = NumberFormatOptions::currency("USD");
        let formatter = NumberFormat::new(Locale::new("en-US").unwrap(), options);

        assert_eq!(formatter.format(1234.56), "$1,234.56");
    }

    #[test]
    fn test_number_format_currency_euro() {
        let options = NumberFormatOptions::currency("EUR");
        let formatter = NumberFormat::new(Locale::new("de-DE").unwrap(), options);

        // German locale puts currency after number
        assert!(formatter.format(1234.56).contains("€"));
    }

    #[test]
    fn test_number_format_percent() {
        let options = NumberFormatOptions::percent();
        let formatter = NumberFormat::new(Locale::new("en-US").unwrap(), options);

        assert_eq!(formatter.format(0.75), "75%");
    }

    #[test]
    fn test_number_format_compact() {
        let options = NumberFormatOptions::compact();
        let formatter = NumberFormat::new(Locale::new("en-US").unwrap(), options);

        assert_eq!(formatter.format(1234.0), "1.23K");
        assert_eq!(formatter.format(1234567.0), "1.23M");
    }

    #[test]
    fn test_number_format_scientific() {
        let options = NumberFormatOptions {
            notation: Notation::Scientific,
            ..Default::default()
        };
        let formatter = NumberFormat::new(Locale::new("en-US").unwrap(), options);

        assert!(formatter.format(1234.0).contains("E"));
    }

    #[test]
    fn test_number_format_sign_always() {
        let options = NumberFormatOptions {
            sign_display: SignDisplay::Always,
            ..Default::default()
        };
        let formatter = NumberFormat::new(Locale::new("en-US").unwrap(), options);

        assert!(formatter.format(5.0).starts_with('+'));
    }

    #[test]
    fn test_number_format_special_values() {
        let formatter = NumberFormat::with_locale(Locale::default());

        assert_eq!(formatter.format(f64::NAN), "NaN");
        assert_eq!(formatter.format(f64::INFINITY), "∞");
        assert_eq!(formatter.format(f64::NEG_INFINITY), "-∞");
    }

    // ==================== DateTimeFormat Tests ====================

    #[test]
    fn test_datetime_format_basic() {
        let options = DateTimeFormatOptions::date_only(DateTimeStyle::Long);
        let formatter = DateTimeFormat::new(Locale::new("en-US").unwrap(), options);

        // Jan 1, 2021 00:00:00 UTC
        let timestamp = 1609459200000.0;
        let formatted = formatter.format(timestamp);

        assert!(formatted.contains("2021"));
        assert!(formatted.contains("January"));
    }

    #[test]
    fn test_datetime_format_short() {
        let options = DateTimeFormatOptions::date_only(DateTimeStyle::Short);
        let formatter = DateTimeFormat::new(Locale::new("en-US").unwrap(), options);

        let timestamp = 1609459200000.0;
        let formatted = formatter.format(timestamp);

        // Should be numeric format like "1/1/21"
        assert!(formatted.contains("/"));
    }

    #[test]
    fn test_datetime_format_invalid() {
        let formatter = DateTimeFormat::with_locale(Locale::default());

        assert_eq!(formatter.format(f64::NAN), "Invalid Date");
    }

    // ==================== PluralRules Tests ====================

    #[test]
    fn test_plural_rules_english_cardinal() {
        let rules = PluralRules::with_locale(Locale::new("en").unwrap());

        assert_eq!(rules.select(0.0), PluralCategory::Other);
        assert_eq!(rules.select(1.0), PluralCategory::One);
        assert_eq!(rules.select(2.0), PluralCategory::Other);
        assert_eq!(rules.select(5.0), PluralCategory::Other);
    }

    #[test]
    fn test_plural_rules_english_ordinal() {
        let options = PluralRulesOptions {
            plural_type: PluralRulesType::Ordinal,
            ..Default::default()
        };
        let rules = PluralRules::new(Locale::new("en").unwrap(), options);

        assert_eq!(rules.select(1.0), PluralCategory::One);   // 1st
        assert_eq!(rules.select(2.0), PluralCategory::Two);   // 2nd
        assert_eq!(rules.select(3.0), PluralCategory::Few);   // 3rd
        assert_eq!(rules.select(4.0), PluralCategory::Other); // 4th
        assert_eq!(rules.select(11.0), PluralCategory::Other); // 11th
        assert_eq!(rules.select(21.0), PluralCategory::One);  // 21st
    }

    #[test]
    fn test_plural_rules_russian() {
        let rules = PluralRules::with_locale(Locale::new("ru").unwrap());

        assert_eq!(rules.select(1.0), PluralCategory::One);
        assert_eq!(rules.select(2.0), PluralCategory::Few);
        assert_eq!(rules.select(5.0), PluralCategory::Many);
        assert_eq!(rules.select(21.0), PluralCategory::One);
        assert_eq!(rules.select(22.0), PluralCategory::Few);
        assert_eq!(rules.select(25.0), PluralCategory::Many);
    }

    // ==================== RelativeTimeFormat Tests ====================

    #[test]
    fn test_relative_time_format_future() {
        let formatter = RelativeTimeFormat::with_locale(Locale::new("en").unwrap());

        assert_eq!(formatter.format(1.0, RelativeTimeUnit::Day), "in 1 day");
        assert_eq!(formatter.format(2.0, RelativeTimeUnit::Day), "in 2 days");
    }

    #[test]
    fn test_relative_time_format_past() {
        let formatter = RelativeTimeFormat::with_locale(Locale::new("en").unwrap());

        assert_eq!(formatter.format(-1.0, RelativeTimeUnit::Day), "1 day ago");
        assert_eq!(formatter.format(-2.0, RelativeTimeUnit::Hour), "2 hours ago");
    }

    #[test]
    fn test_relative_time_format_auto() {
        let options = RelativeTimeFormatOptions {
            numeric: RelativeTimeNumeric::Auto,
            ..Default::default()
        };
        let formatter = RelativeTimeFormat::new(Locale::new("en").unwrap(), options);

        assert_eq!(formatter.format(-1.0, RelativeTimeUnit::Day), "yesterday");
        assert_eq!(formatter.format(1.0, RelativeTimeUnit::Day), "tomorrow");
    }

    #[test]
    fn test_relative_time_format_narrow() {
        let options = RelativeTimeFormatOptions {
            style: RelativeTimeStyle::Narrow,
            ..Default::default()
        };
        let formatter = RelativeTimeFormat::new(Locale::new("en").unwrap(), options);

        assert_eq!(formatter.format(5.0, RelativeTimeUnit::Day), "in 5 d");
    }

    // ==================== ListFormat Tests ====================

    #[test]
    fn test_list_format_conjunction() {
        let formatter = ListFormat::with_locale(Locale::new("en-US").unwrap());

        assert_eq!(formatter.format_str(&["a"]), "a");
        assert_eq!(formatter.format_str(&["a", "b"]), "a and b");
        assert_eq!(formatter.format_str(&["a", "b", "c"]), "a, b, and c");
    }

    #[test]
    fn test_list_format_disjunction() {
        let options = ListFormatOptions {
            list_type: ListFormatType::Disjunction,
            ..Default::default()
        };
        let formatter = ListFormat::new(Locale::new("en-US").unwrap(), options);

        assert_eq!(formatter.format_str(&["a", "b"]), "a or b");
        assert_eq!(formatter.format_str(&["a", "b", "c"]), "a, b, or c");
    }

    #[test]
    fn test_list_format_short() {
        let options = ListFormatOptions {
            style: ListFormatStyle::Short,
            ..Default::default()
        };
        let formatter = ListFormat::new(Locale::new("en").unwrap(), options);

        assert_eq!(formatter.format_str(&["a", "b"]), "a & b");
    }

    #[test]
    fn test_list_format_empty() {
        let formatter = ListFormat::with_locale(Locale::default());

        assert_eq!(formatter.format(&[]), "");
    }

    #[test]
    fn test_list_format_british() {
        let formatter = ListFormat::with_locale(Locale::new("en-GB").unwrap());

        // British English doesn't use Oxford comma
        assert_eq!(formatter.format_str(&["a", "b", "c"]), "a, b and c");
    }

    // ==================== Intl Tests ====================

    #[test]
    fn test_intl_get_canonical_locales() {
        let result = Intl::get_canonical_locales(&["en-us", "DE-de"]).unwrap();

        assert_eq!(result[0], "en-US");
        assert_eq!(result[1], "de-DE");
    }

    #[test]
    fn test_intl_get_canonical_locales_invalid() {
        let result = Intl::get_canonical_locales(&["invalid"]);
        assert!(result.is_err());
    }
}

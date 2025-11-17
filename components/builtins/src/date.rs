//! ECMAScript Date object implementation
//!
//! Implements the Date object per ES2024 specification with:
//! - Multiple constructor forms
//! - Getters and setters for date/time components
//! - Formatting methods
//! - Static methods (now, parse, UTC)

use chrono::{
    Datelike, Local, NaiveDate, NaiveDateTime, TimeZone, Timelike, Utc, FixedOffset,
};
use std::cell::RefCell;
use std::rc::Rc;

/// JavaScript Date object representation
#[derive(Debug, Clone)]
pub struct DateObject {
    /// Internal time value in milliseconds since Unix epoch (January 1, 1970 00:00:00 UTC)
    /// NaN represents an invalid date
    time_value: f64,
}

/// Wrapper for Date as a JsValue-like type
#[derive(Debug, Clone)]
pub struct JsDate {
    inner: Rc<RefCell<DateObject>>,
}

impl JsDate {
    /// Create a new Date with current time
    pub fn new() -> Self {
        let now = Utc::now().timestamp_millis() as f64;
        JsDate {
            inner: Rc::new(RefCell::new(DateObject { time_value: now })),
        }
    }

    /// Create a Date from milliseconds since epoch
    pub fn from_timestamp(ms: f64) -> Self {
        let time_value = if ms.is_nan() || ms.is_infinite() {
            f64::NAN
        } else {
            ms.trunc()
        };
        JsDate {
            inner: Rc::new(RefCell::new(DateObject { time_value })),
        }
    }

    /// Create a Date from date components (year, month, day, hours, minutes, seconds, ms)
    ///
    /// Month is 0-indexed (0 = January, 11 = December)
    /// Years 0-99 map to 1900-1999
    pub fn from_components(
        year: i32,
        month: u32,
        day: Option<u32>,
        hours: Option<u32>,
        minutes: Option<u32>,
        seconds: Option<u32>,
        ms: Option<u32>,
    ) -> Self {
        // Handle two-digit year (0-99 maps to 1900-1999)
        let actual_year = if year >= 0 && year <= 99 {
            1900 + year
        } else {
            year
        };

        let day = day.unwrap_or(1);
        let hours = hours.unwrap_or(0);
        let minutes = minutes.unwrap_or(0);
        let seconds = seconds.unwrap_or(0);
        let ms = ms.unwrap_or(0);

        // Month in JS is 0-indexed, chrono needs 1-indexed
        let chrono_month = (month + 1) as u32;

        let result = NaiveDate::from_ymd_opt(actual_year, chrono_month, day).and_then(|date| {
            date.and_hms_milli_opt(hours, minutes, seconds, ms)
        });

        match result {
            Some(dt) => {
                // Convert local time to UTC timestamp
                let local_result = Local.from_local_datetime(&dt);
                match local_result.single() {
                    Some(local_dt) => {
                        let utc_ms = local_dt.timestamp_millis() as f64;
                        JsDate {
                            inner: Rc::new(RefCell::new(DateObject {
                                time_value: utc_ms,
                            })),
                        }
                    }
                    None => JsDate::invalid(),
                }
            }
            None => JsDate::invalid(),
        }
    }

    /// Parse an ISO 8601 date string
    pub fn from_string(s: &str) -> Self {
        let time_value = DateConstructor::parse(s);
        JsDate {
            inner: Rc::new(RefCell::new(DateObject { time_value })),
        }
    }

    /// Create an invalid date (NaN time value)
    pub fn invalid() -> Self {
        JsDate {
            inner: Rc::new(RefCell::new(DateObject {
                time_value: f64::NAN,
            })),
        }
    }

    /// Get the internal time value (milliseconds since epoch)
    pub fn get_time(&self) -> f64 {
        self.inner.borrow().time_value
    }

    /// Set the internal time value
    pub fn set_time(&self, ms: f64) -> f64 {
        let time_value = if ms.is_nan() || ms.is_infinite() {
            f64::NAN
        } else {
            ms.trunc()
        };
        self.inner.borrow_mut().time_value = time_value;
        time_value
    }

    /// Check if the date is valid (not NaN)
    pub fn is_valid(&self) -> bool {
        !self.inner.borrow().time_value.is_nan()
    }

    // Helper to get NaiveDateTime in UTC
    fn to_utc_datetime(&self) -> Option<NaiveDateTime> {
        let ms = self.inner.borrow().time_value;
        if ms.is_nan() {
            return None;
        }
        let secs = (ms / 1000.0).floor() as i64;
        let nsecs = ((ms % 1000.0) * 1_000_000.0) as u32;
        chrono::DateTime::from_timestamp(secs, nsecs).map(|dt| dt.naive_utc())
    }

    // Helper to get DateTime in local timezone
    fn to_local_datetime(&self) -> Option<chrono::DateTime<Local>> {
        self.to_utc_datetime()
            .map(|dt| Local.from_utc_datetime(&dt))
    }

    // ===== GETTERS (Local Time) =====

    /// Get the full year (e.g., 2024)
    pub fn get_full_year(&self) -> f64 {
        self.to_local_datetime()
            .map(|dt| dt.year() as f64)
            .unwrap_or(f64::NAN)
    }

    /// Get the month (0-11)
    pub fn get_month(&self) -> f64 {
        self.to_local_datetime()
            .map(|dt| (dt.month() - 1) as f64)
            .unwrap_or(f64::NAN)
    }

    /// Get the day of the month (1-31)
    pub fn get_date(&self) -> f64 {
        self.to_local_datetime()
            .map(|dt| dt.day() as f64)
            .unwrap_or(f64::NAN)
    }

    /// Get the day of the week (0 = Sunday, 6 = Saturday)
    pub fn get_day(&self) -> f64 {
        self.to_local_datetime()
            .map(|dt| {
                // chrono: Monday = 0, ..., Sunday = 6
                // JS: Sunday = 0, ..., Saturday = 6
                let chrono_day = dt.weekday().num_days_from_sunday();
                chrono_day as f64
            })
            .unwrap_or(f64::NAN)
    }

    /// Get the hours (0-23)
    pub fn get_hours(&self) -> f64 {
        self.to_local_datetime()
            .map(|dt| dt.hour() as f64)
            .unwrap_or(f64::NAN)
    }

    /// Get the minutes (0-59)
    pub fn get_minutes(&self) -> f64 {
        self.to_local_datetime()
            .map(|dt| dt.minute() as f64)
            .unwrap_or(f64::NAN)
    }

    /// Get the seconds (0-59)
    pub fn get_seconds(&self) -> f64 {
        self.to_local_datetime()
            .map(|dt| dt.second() as f64)
            .unwrap_or(f64::NAN)
    }

    /// Get the milliseconds (0-999)
    pub fn get_milliseconds(&self) -> f64 {
        let ms = self.inner.borrow().time_value;
        if ms.is_nan() {
            f64::NAN
        } else {
            (ms % 1000.0).abs()
        }
    }

    /// Get timezone offset in minutes (positive means behind UTC, negative means ahead)
    pub fn get_timezone_offset(&self) -> f64 {
        self.to_local_datetime()
            .map(|dt| {
                let offset_seconds = dt.offset().local_minus_utc();
                // JavaScript returns the opposite sign
                -(offset_seconds / 60) as f64
            })
            .unwrap_or(f64::NAN)
    }

    // ===== GETTERS (UTC) =====

    /// Get the UTC full year
    pub fn get_utc_full_year(&self) -> f64 {
        self.to_utc_datetime()
            .map(|dt| dt.year() as f64)
            .unwrap_or(f64::NAN)
    }

    /// Get the UTC month (0-11)
    pub fn get_utc_month(&self) -> f64 {
        self.to_utc_datetime()
            .map(|dt| (dt.month() - 1) as f64)
            .unwrap_or(f64::NAN)
    }

    /// Get the UTC day of month (1-31)
    pub fn get_utc_date(&self) -> f64 {
        self.to_utc_datetime()
            .map(|dt| dt.day() as f64)
            .unwrap_or(f64::NAN)
    }

    /// Get the UTC day of week (0 = Sunday, 6 = Saturday)
    pub fn get_utc_day(&self) -> f64 {
        self.to_utc_datetime()
            .map(|dt| dt.weekday().num_days_from_sunday() as f64)
            .unwrap_or(f64::NAN)
    }

    /// Get the UTC hours (0-23)
    pub fn get_utc_hours(&self) -> f64 {
        self.to_utc_datetime()
            .map(|dt| dt.hour() as f64)
            .unwrap_or(f64::NAN)
    }

    /// Get the UTC minutes (0-59)
    pub fn get_utc_minutes(&self) -> f64 {
        self.to_utc_datetime()
            .map(|dt| dt.minute() as f64)
            .unwrap_or(f64::NAN)
    }

    /// Get the UTC seconds (0-59)
    pub fn get_utc_seconds(&self) -> f64 {
        self.to_utc_datetime()
            .map(|dt| dt.second() as f64)
            .unwrap_or(f64::NAN)
    }

    /// Get the UTC milliseconds (0-999)
    pub fn get_utc_milliseconds(&self) -> f64 {
        self.get_milliseconds()
    }

    // ===== SETTERS (Local Time) =====

    /// Set the full year (and optionally month and date)
    pub fn set_full_year(&self, year: i32, month: Option<u32>, date: Option<u32>) -> f64 {
        if let Some(dt) = self.to_local_datetime() {
            let new_month = month.unwrap_or(dt.month() - 1) + 1;
            let new_date = date.unwrap_or(dt.day());
            let new_dt = NaiveDate::from_ymd_opt(year, new_month, new_date).and_then(|d| {
                d.and_hms_milli_opt(dt.hour(), dt.minute(), dt.second(), self.get_milliseconds() as u32)
            });
            if let Some(naive_dt) = new_dt {
                if let Some(local_dt) = Local.from_local_datetime(&naive_dt).single() {
                    return self.set_time(local_dt.timestamp_millis() as f64);
                }
            }
        }
        self.set_time(f64::NAN)
    }

    /// Set the month (0-11) and optionally the date
    pub fn set_month(&self, month: u32, date: Option<u32>) -> f64 {
        if let Some(dt) = self.to_local_datetime() {
            let new_date = date.unwrap_or(dt.day());
            let new_dt = NaiveDate::from_ymd_opt(dt.year(), month + 1, new_date).and_then(|d| {
                d.and_hms_milli_opt(dt.hour(), dt.minute(), dt.second(), self.get_milliseconds() as u32)
            });
            if let Some(naive_dt) = new_dt {
                if let Some(local_dt) = Local.from_local_datetime(&naive_dt).single() {
                    return self.set_time(local_dt.timestamp_millis() as f64);
                }
            }
        }
        self.set_time(f64::NAN)
    }

    /// Set the day of the month (1-31)
    pub fn set_date(&self, date: u32) -> f64 {
        if let Some(dt) = self.to_local_datetime() {
            let new_dt = NaiveDate::from_ymd_opt(dt.year(), dt.month(), date).and_then(|d| {
                d.and_hms_milli_opt(dt.hour(), dt.minute(), dt.second(), self.get_milliseconds() as u32)
            });
            if let Some(naive_dt) = new_dt {
                if let Some(local_dt) = Local.from_local_datetime(&naive_dt).single() {
                    return self.set_time(local_dt.timestamp_millis() as f64);
                }
            }
        }
        self.set_time(f64::NAN)
    }

    /// Set the hours (and optionally minutes, seconds, ms)
    pub fn set_hours(
        &self,
        hours: u32,
        minutes: Option<u32>,
        seconds: Option<u32>,
        ms: Option<u32>,
    ) -> f64 {
        if let Some(dt) = self.to_local_datetime() {
            let new_minutes = minutes.unwrap_or(dt.minute());
            let new_seconds = seconds.unwrap_or(dt.second());
            let new_ms = ms.unwrap_or(self.get_milliseconds() as u32);
            let new_dt = NaiveDate::from_ymd_opt(dt.year(), dt.month(), dt.day())
                .and_then(|d| d.and_hms_milli_opt(hours, new_minutes, new_seconds, new_ms));
            if let Some(naive_dt) = new_dt {
                if let Some(local_dt) = Local.from_local_datetime(&naive_dt).single() {
                    return self.set_time(local_dt.timestamp_millis() as f64);
                }
            }
        }
        self.set_time(f64::NAN)
    }

    /// Set the minutes (and optionally seconds and ms)
    pub fn set_minutes(&self, minutes: u32, seconds: Option<u32>, ms: Option<u32>) -> f64 {
        if let Some(dt) = self.to_local_datetime() {
            let new_seconds = seconds.unwrap_or(dt.second());
            let new_ms = ms.unwrap_or(self.get_milliseconds() as u32);
            let new_dt = NaiveDate::from_ymd_opt(dt.year(), dt.month(), dt.day())
                .and_then(|d| d.and_hms_milli_opt(dt.hour(), minutes, new_seconds, new_ms));
            if let Some(naive_dt) = new_dt {
                if let Some(local_dt) = Local.from_local_datetime(&naive_dt).single() {
                    return self.set_time(local_dt.timestamp_millis() as f64);
                }
            }
        }
        self.set_time(f64::NAN)
    }

    /// Set the seconds (and optionally ms)
    pub fn set_seconds(&self, seconds: u32, ms: Option<u32>) -> f64 {
        if let Some(dt) = self.to_local_datetime() {
            let new_ms = ms.unwrap_or(self.get_milliseconds() as u32);
            let new_dt = NaiveDate::from_ymd_opt(dt.year(), dt.month(), dt.day())
                .and_then(|d| d.and_hms_milli_opt(dt.hour(), dt.minute(), seconds, new_ms));
            if let Some(naive_dt) = new_dt {
                if let Some(local_dt) = Local.from_local_datetime(&naive_dt).single() {
                    return self.set_time(local_dt.timestamp_millis() as f64);
                }
            }
        }
        self.set_time(f64::NAN)
    }

    /// Set the milliseconds
    pub fn set_milliseconds(&self, ms: u32) -> f64 {
        if let Some(dt) = self.to_local_datetime() {
            let new_dt = NaiveDate::from_ymd_opt(dt.year(), dt.month(), dt.day())
                .and_then(|d| d.and_hms_milli_opt(dt.hour(), dt.minute(), dt.second(), ms));
            if let Some(naive_dt) = new_dt {
                if let Some(local_dt) = Local.from_local_datetime(&naive_dt).single() {
                    return self.set_time(local_dt.timestamp_millis() as f64);
                }
            }
        }
        self.set_time(f64::NAN)
    }

    // ===== SETTERS (UTC) =====

    /// Set the UTC full year (and optionally month and date)
    pub fn set_utc_full_year(&self, year: i32, month: Option<u32>, date: Option<u32>) -> f64 {
        if let Some(dt) = self.to_utc_datetime() {
            let new_month = month.unwrap_or(dt.month() - 1) + 1;
            let new_date = date.unwrap_or(dt.day());
            let new_dt = NaiveDate::from_ymd_opt(year, new_month, new_date).and_then(|d| {
                d.and_hms_milli_opt(dt.hour(), dt.minute(), dt.second(), self.get_milliseconds() as u32)
            });
            if let Some(naive_dt) = new_dt {
                let utc_dt = Utc.from_utc_datetime(&naive_dt);
                return self.set_time(utc_dt.timestamp_millis() as f64);
            }
        }
        self.set_time(f64::NAN)
    }

    /// Set the UTC month (0-11) and optionally the date
    pub fn set_utc_month(&self, month: u32, date: Option<u32>) -> f64 {
        if let Some(dt) = self.to_utc_datetime() {
            let new_date = date.unwrap_or(dt.day());
            let new_dt = NaiveDate::from_ymd_opt(dt.year(), month + 1, new_date).and_then(|d| {
                d.and_hms_milli_opt(dt.hour(), dt.minute(), dt.second(), self.get_milliseconds() as u32)
            });
            if let Some(naive_dt) = new_dt {
                let utc_dt = Utc.from_utc_datetime(&naive_dt);
                return self.set_time(utc_dt.timestamp_millis() as f64);
            }
        }
        self.set_time(f64::NAN)
    }

    /// Set the UTC day of the month (1-31)
    pub fn set_utc_date(&self, date: u32) -> f64 {
        if let Some(dt) = self.to_utc_datetime() {
            let new_dt = NaiveDate::from_ymd_opt(dt.year(), dt.month(), date).and_then(|d| {
                d.and_hms_milli_opt(dt.hour(), dt.minute(), dt.second(), self.get_milliseconds() as u32)
            });
            if let Some(naive_dt) = new_dt {
                let utc_dt = Utc.from_utc_datetime(&naive_dt);
                return self.set_time(utc_dt.timestamp_millis() as f64);
            }
        }
        self.set_time(f64::NAN)
    }

    /// Set the UTC hours (and optionally minutes, seconds, ms)
    pub fn set_utc_hours(
        &self,
        hours: u32,
        minutes: Option<u32>,
        seconds: Option<u32>,
        ms: Option<u32>,
    ) -> f64 {
        if let Some(dt) = self.to_utc_datetime() {
            let new_minutes = minutes.unwrap_or(dt.minute());
            let new_seconds = seconds.unwrap_or(dt.second());
            let new_ms = ms.unwrap_or(self.get_milliseconds() as u32);
            let new_dt = NaiveDate::from_ymd_opt(dt.year(), dt.month(), dt.day())
                .and_then(|d| d.and_hms_milli_opt(hours, new_minutes, new_seconds, new_ms));
            if let Some(naive_dt) = new_dt {
                let utc_dt = Utc.from_utc_datetime(&naive_dt);
                return self.set_time(utc_dt.timestamp_millis() as f64);
            }
        }
        self.set_time(f64::NAN)
    }

    /// Set the UTC minutes (and optionally seconds and ms)
    pub fn set_utc_minutes(&self, minutes: u32, seconds: Option<u32>, ms: Option<u32>) -> f64 {
        if let Some(dt) = self.to_utc_datetime() {
            let new_seconds = seconds.unwrap_or(dt.second());
            let new_ms = ms.unwrap_or(self.get_milliseconds() as u32);
            let new_dt = NaiveDate::from_ymd_opt(dt.year(), dt.month(), dt.day())
                .and_then(|d| d.and_hms_milli_opt(dt.hour(), minutes, new_seconds, new_ms));
            if let Some(naive_dt) = new_dt {
                let utc_dt = Utc.from_utc_datetime(&naive_dt);
                return self.set_time(utc_dt.timestamp_millis() as f64);
            }
        }
        self.set_time(f64::NAN)
    }

    /// Set the UTC seconds (and optionally ms)
    pub fn set_utc_seconds(&self, seconds: u32, ms: Option<u32>) -> f64 {
        if let Some(dt) = self.to_utc_datetime() {
            let new_ms = ms.unwrap_or(self.get_milliseconds() as u32);
            let new_dt = NaiveDate::from_ymd_opt(dt.year(), dt.month(), dt.day())
                .and_then(|d| d.and_hms_milli_opt(dt.hour(), dt.minute(), seconds, new_ms));
            if let Some(naive_dt) = new_dt {
                let utc_dt = Utc.from_utc_datetime(&naive_dt);
                return self.set_time(utc_dt.timestamp_millis() as f64);
            }
        }
        self.set_time(f64::NAN)
    }

    /// Set the UTC milliseconds
    pub fn set_utc_milliseconds(&self, ms: u32) -> f64 {
        if let Some(dt) = self.to_utc_datetime() {
            let new_dt = NaiveDate::from_ymd_opt(dt.year(), dt.month(), dt.day())
                .and_then(|d| d.and_hms_milli_opt(dt.hour(), dt.minute(), dt.second(), ms));
            if let Some(naive_dt) = new_dt {
                let utc_dt = Utc.from_utc_datetime(&naive_dt);
                return self.set_time(utc_dt.timestamp_millis() as f64);
            }
        }
        self.set_time(f64::NAN)
    }

    // ===== FORMATTING METHODS =====

    /// Convert to string (like JavaScript's Date.prototype.toString)
    pub fn to_string(&self) -> String {
        if let Some(dt) = self.to_local_datetime() {
            // Format: "Tue Nov 05 2024 14:30:00 GMT+0000 (Coordinated Universal Time)"
            let day_name = match dt.weekday() {
                chrono::Weekday::Mon => "Mon",
                chrono::Weekday::Tue => "Tue",
                chrono::Weekday::Wed => "Wed",
                chrono::Weekday::Thu => "Thu",
                chrono::Weekday::Fri => "Fri",
                chrono::Weekday::Sat => "Sat",
                chrono::Weekday::Sun => "Sun",
            };
            let month_name = match dt.month() {
                1 => "Jan",
                2 => "Feb",
                3 => "Mar",
                4 => "Apr",
                5 => "May",
                6 => "Jun",
                7 => "Jul",
                8 => "Aug",
                9 => "Sep",
                10 => "Oct",
                11 => "Nov",
                12 => "Dec",
                _ => "???",
            };
            let offset_seconds = dt.offset().local_minus_utc();
            let offset_hours = offset_seconds.abs() / 3600;
            let offset_minutes = (offset_seconds.abs() % 3600) / 60;
            let offset_sign = if offset_seconds >= 0 { '+' } else { '-' };

            format!(
                "{} {} {:02} {:04} {:02}:{:02}:{:02} GMT{}{:02}{:02}",
                day_name,
                month_name,
                dt.day(),
                dt.year(),
                dt.hour(),
                dt.minute(),
                dt.second(),
                offset_sign,
                offset_hours,
                offset_minutes
            )
        } else {
            "Invalid Date".to_string()
        }
    }

    /// Convert to date string (like JavaScript's Date.prototype.toDateString)
    pub fn to_date_string(&self) -> String {
        if let Some(dt) = self.to_local_datetime() {
            let day_name = match dt.weekday() {
                chrono::Weekday::Mon => "Mon",
                chrono::Weekday::Tue => "Tue",
                chrono::Weekday::Wed => "Wed",
                chrono::Weekday::Thu => "Thu",
                chrono::Weekday::Fri => "Fri",
                chrono::Weekday::Sat => "Sat",
                chrono::Weekday::Sun => "Sun",
            };
            let month_name = match dt.month() {
                1 => "Jan",
                2 => "Feb",
                3 => "Mar",
                4 => "Apr",
                5 => "May",
                6 => "Jun",
                7 => "Jul",
                8 => "Aug",
                9 => "Sep",
                10 => "Oct",
                11 => "Nov",
                12 => "Dec",
                _ => "???",
            };
            format!("{} {} {:02} {:04}", day_name, month_name, dt.day(), dt.year())
        } else {
            "Invalid Date".to_string()
        }
    }

    /// Convert to time string (like JavaScript's Date.prototype.toTimeString)
    pub fn to_time_string(&self) -> String {
        if let Some(dt) = self.to_local_datetime() {
            let offset_seconds = dt.offset().local_minus_utc();
            let offset_hours = offset_seconds.abs() / 3600;
            let offset_minutes = (offset_seconds.abs() % 3600) / 60;
            let offset_sign = if offset_seconds >= 0 { '+' } else { '-' };

            format!(
                "{:02}:{:02}:{:02} GMT{}{:02}{:02}",
                dt.hour(),
                dt.minute(),
                dt.second(),
                offset_sign,
                offset_hours,
                offset_minutes
            )
        } else {
            "Invalid Date".to_string()
        }
    }

    /// Convert to ISO 8601 string (like JavaScript's Date.prototype.toISOString)
    pub fn to_iso_string(&self) -> Result<String, String> {
        if let Some(dt) = self.to_utc_datetime() {
            Ok(format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
                dt.year(),
                dt.month(),
                dt.day(),
                dt.hour(),
                dt.minute(),
                dt.second(),
                (self.get_milliseconds() as u32)
            ))
        } else {
            Err("Invalid Date".to_string())
        }
    }

    /// Convert to JSON (same as toISOString or null for invalid)
    pub fn to_json(&self) -> Option<String> {
        self.to_iso_string().ok()
    }

    /// Convert to locale string (simplified implementation)
    pub fn to_locale_string(&self) -> String {
        if let Some(dt) = self.to_local_datetime() {
            format!(
                "{}/{}/{}, {:02}:{:02}:{:02}",
                dt.month(),
                dt.day(),
                dt.year(),
                dt.hour(),
                dt.minute(),
                dt.second()
            )
        } else {
            "Invalid Date".to_string()
        }
    }

    /// Convert to locale date string (simplified implementation)
    pub fn to_locale_date_string(&self) -> String {
        if let Some(dt) = self.to_local_datetime() {
            format!("{}/{}/{}", dt.month(), dt.day(), dt.year())
        } else {
            "Invalid Date".to_string()
        }
    }

    /// Convert to locale time string (simplified implementation)
    pub fn to_locale_time_string(&self) -> String {
        if let Some(dt) = self.to_local_datetime() {
            format!("{:02}:{:02}:{:02}", dt.hour(), dt.minute(), dt.second())
        } else {
            "Invalid Date".to_string()
        }
    }

    /// Get the primitive value (same as getTime)
    pub fn value_of(&self) -> f64 {
        self.get_time()
    }
}

impl Default for JsDate {
    fn default() -> Self {
        JsDate::new()
    }
}

/// Date constructor providing static methods
pub struct DateConstructor;

impl DateConstructor {
    /// Get current timestamp in milliseconds (Date.now())
    pub fn now() -> f64 {
        Utc::now().timestamp_millis() as f64
    }

    /// Parse a date string and return milliseconds since epoch (Date.parse)
    ///
    /// Supports ISO 8601 format: YYYY-MM-DDTHH:mm:ss.sssZ
    pub fn parse(s: &str) -> f64 {
        let s = s.trim();

        // Try ISO 8601 format first: YYYY-MM-DDTHH:mm:ss.sssZ
        if let Some(ts) = Self::parse_iso8601(s) {
            return ts;
        }

        // Try common date formats
        if let Some(ts) = Self::parse_common_formats(s) {
            return ts;
        }

        f64::NAN
    }

    fn parse_iso8601(s: &str) -> Option<f64> {
        // Full ISO: 2024-11-05T14:30:00.000Z
        // Date only: 2024-11-05
        // With offset: 2024-11-05T14:30:00+05:30

        if s.contains('T') || s.contains(' ') {
            // Date and time
            let (date_part, time_part) = if s.contains('T') {
                let parts: Vec<&str> = s.splitn(2, 'T').collect();
                if parts.len() != 2 {
                    return None;
                }
                (parts[0], parts[1])
            } else {
                let parts: Vec<&str> = s.splitn(2, ' ').collect();
                if parts.len() != 2 {
                    return None;
                }
                (parts[0], parts[1])
            };

            let date_components: Vec<i32> = date_part
                .split('-')
                .filter_map(|p| p.parse().ok())
                .collect();

            if date_components.len() < 3 {
                return None;
            }

            let year = date_components[0];
            let month = date_components[1] as u32;
            let day = date_components[2] as u32;

            // Parse time part
            let (time_str, offset_str) = if time_part.contains('Z') {
                (time_part.trim_end_matches('Z'), Some("+00:00"))
            } else if time_part.contains('+') {
                let parts: Vec<&str> = time_part.splitn(2, '+').collect();
                (parts[0], Some(&time_part[parts[0].len()..]))
            } else if time_part.matches('-').count() > 0
                && time_part.rfind('-').unwrap() > time_part.find(':').unwrap_or(0)
            {
                let idx = time_part.rfind('-').unwrap();
                (&time_part[..idx], Some(&time_part[idx..]))
            } else {
                (time_part, None)
            };

            let time_components: Vec<&str> = time_str.split(':').collect();
            let hours: u32 = time_components.get(0)?.parse().ok()?;
            let minutes: u32 = time_components.get(1).unwrap_or(&"0").parse().ok()?;

            let seconds_and_ms = time_components.get(2).unwrap_or(&"0");
            let (seconds, ms) = if seconds_and_ms.contains('.') {
                let parts: Vec<&str> = seconds_and_ms.split('.').collect();
                let secs: u32 = parts[0].parse().ok()?;
                let ms_str = parts.get(1).unwrap_or(&"0");
                let ms: u32 = if ms_str.len() >= 3 {
                    ms_str[..3].parse().ok()?
                } else {
                    let padded = format!("{:0<3}", ms_str);
                    padded.parse().ok()?
                };
                (secs, ms)
            } else {
                (seconds_and_ms.parse().ok()?, 0)
            };

            let naive_dt = NaiveDate::from_ymd_opt(year, month, day)?
                .and_hms_milli_opt(hours, minutes, seconds, ms)?;

            let timestamp = if let Some(offset) = offset_str {
                // Parse timezone offset
                let offset_cleaned = offset.trim_start_matches('+').trim_start_matches('-');
                let is_negative = offset.starts_with('-');

                let offset_parts: Vec<&str> = offset_cleaned.split(':').collect();
                let offset_hours: i32 = offset_parts.get(0)?.parse().ok()?;
                let offset_minutes: i32 = offset_parts.get(1).unwrap_or(&"0").parse().ok()?;

                let total_offset_seconds = if is_negative {
                    -(offset_hours * 3600 + offset_minutes * 60)
                } else {
                    offset_hours * 3600 + offset_minutes * 60
                };

                let fixed_offset = FixedOffset::east_opt(total_offset_seconds)?;
                let dt_with_offset = fixed_offset.from_local_datetime(&naive_dt).single()?;
                dt_with_offset.timestamp_millis() as f64
            } else {
                // No offset specified - treat as UTC for ISO format
                let utc_dt = Utc.from_utc_datetime(&naive_dt);
                utc_dt.timestamp_millis() as f64
            };

            Some(timestamp)
        } else {
            // Date only: YYYY-MM-DD
            let parts: Vec<i32> = s.split('-').filter_map(|p| p.parse().ok()).collect();
            if parts.len() >= 3 {
                let year = parts[0];
                let month = parts[1] as u32;
                let day = parts[2] as u32;

                let naive_dt =
                    NaiveDate::from_ymd_opt(year, month, day)?.and_hms_milli_opt(0, 0, 0, 0)?;

                let utc_dt = Utc.from_utc_datetime(&naive_dt);
                Some(utc_dt.timestamp_millis() as f64)
            } else {
                None
            }
        }
    }

    fn parse_common_formats(s: &str) -> Option<f64> {
        // Try to parse formats like "Nov 5, 2024" or "11/5/2024"

        // MM/DD/YYYY format
        if s.contains('/') {
            let parts: Vec<i32> = s.split('/').filter_map(|p| p.trim().parse().ok()).collect();
            if parts.len() == 3 {
                let year = if parts[2] < 100 {
                    if parts[2] < 50 {
                        2000 + parts[2]
                    } else {
                        1900 + parts[2]
                    }
                } else {
                    parts[2]
                };
                let month = parts[0] as u32;
                let day = parts[1] as u32;

                let naive_dt =
                    NaiveDate::from_ymd_opt(year, month, day)?.and_hms_milli_opt(0, 0, 0, 0)?;

                let utc_dt = Utc.from_utc_datetime(&naive_dt);
                return Some(utc_dt.timestamp_millis() as f64);
            }
        }

        None
    }

    /// Create a UTC timestamp from components (Date.UTC)
    ///
    /// Month is 0-indexed (0 = January, 11 = December)
    /// Years 0-99 map to 1900-1999
    pub fn utc(
        year: i32,
        month: u32,
        day: Option<u32>,
        hours: Option<u32>,
        minutes: Option<u32>,
        seconds: Option<u32>,
        ms: Option<u32>,
    ) -> f64 {
        // Handle two-digit year (0-99 maps to 1900-1999)
        let actual_year = if year >= 0 && year <= 99 {
            1900 + year
        } else {
            year
        };

        let day = day.unwrap_or(1);
        let hours = hours.unwrap_or(0);
        let minutes = minutes.unwrap_or(0);
        let seconds = seconds.unwrap_or(0);
        let ms = ms.unwrap_or(0);

        let chrono_month = (month + 1) as u32;

        let result = NaiveDate::from_ymd_opt(actual_year, chrono_month, day)
            .and_then(|date| date.and_hms_milli_opt(hours, minutes, seconds, ms));

        match result {
            Some(dt) => {
                let utc_dt = Utc.from_utc_datetime(&dt);
                utc_dt.timestamp_millis() as f64
            }
            None => f64::NAN,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Constructor Tests =====

    #[test]
    fn test_new_date_current_time() {
        let before = DateConstructor::now();
        let date = JsDate::new();
        let after = DateConstructor::now();

        let time = date.get_time();
        assert!(time >= before);
        assert!(time <= after);
        assert!(date.is_valid());
    }

    #[test]
    fn test_from_timestamp() {
        let timestamp = 1609459200000.0; // 2021-01-01T00:00:00.000Z
        let date = JsDate::from_timestamp(timestamp);

        assert_eq!(date.get_time(), timestamp);
        assert_eq!(date.get_utc_full_year(), 2021.0);
        assert_eq!(date.get_utc_month(), 0.0);
        assert_eq!(date.get_utc_date(), 1.0);
    }

    #[test]
    fn test_from_timestamp_with_nan() {
        let date = JsDate::from_timestamp(f64::NAN);
        assert!(!date.is_valid());
        assert!(date.get_time().is_nan());
    }

    #[test]
    fn test_from_timestamp_with_infinity() {
        let date = JsDate::from_timestamp(f64::INFINITY);
        assert!(!date.is_valid());
    }

    #[test]
    fn test_from_components_basic() {
        // 2024-01-15 (Month is 0-indexed in JS)
        let date = JsDate::from_components(2024, 0, Some(15), None, None, None, None);
        assert!(date.is_valid());
        // Note: The actual values depend on local timezone
        assert_eq!(date.get_full_year(), 2024.0);
        assert_eq!(date.get_month(), 0.0);
        assert_eq!(date.get_date(), 15.0);
    }

    #[test]
    fn test_from_components_with_time() {
        let date = JsDate::from_components(2024, 5, Some(15), Some(14), Some(30), Some(45), Some(123));
        assert!(date.is_valid());
        assert_eq!(date.get_full_year(), 2024.0);
        assert_eq!(date.get_month(), 5.0);
        assert_eq!(date.get_date(), 15.0);
        assert_eq!(date.get_hours(), 14.0);
        assert_eq!(date.get_minutes(), 30.0);
        assert_eq!(date.get_seconds(), 45.0);
        assert_eq!(date.get_milliseconds(), 123.0);
    }

    #[test]
    fn test_from_components_two_digit_year() {
        // Two-digit year should map to 1900-1999
        let date = JsDate::from_components(99, 0, Some(1), None, None, None, None);
        assert!(date.is_valid());
        assert_eq!(date.get_full_year(), 1999.0);

        let date2 = JsDate::from_components(0, 0, Some(1), None, None, None, None);
        assert_eq!(date2.get_full_year(), 1900.0);
    }

    #[test]
    fn test_from_string_iso8601() {
        let date = JsDate::from_string("2024-11-05T14:30:00.000Z");
        assert!(date.is_valid());
        assert_eq!(date.get_utc_full_year(), 2024.0);
        assert_eq!(date.get_utc_month(), 10.0); // November = 10 (0-indexed)
        assert_eq!(date.get_utc_date(), 5.0);
        assert_eq!(date.get_utc_hours(), 14.0);
        assert_eq!(date.get_utc_minutes(), 30.0);
    }

    #[test]
    fn test_from_string_date_only() {
        let date = JsDate::from_string("2024-11-05");
        assert!(date.is_valid());
        assert_eq!(date.get_utc_full_year(), 2024.0);
        assert_eq!(date.get_utc_month(), 10.0);
        assert_eq!(date.get_utc_date(), 5.0);
    }

    #[test]
    fn test_from_string_invalid() {
        let date = JsDate::from_string("not a date");
        assert!(!date.is_valid());
    }

    #[test]
    fn test_invalid_date() {
        let date = JsDate::invalid();
        assert!(!date.is_valid());
        assert!(date.get_time().is_nan());
        assert!(date.get_full_year().is_nan());
        assert!(date.get_month().is_nan());
    }

    // ===== Static Method Tests =====

    #[test]
    fn test_date_now() {
        let before = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as f64;

        let now = DateConstructor::now();

        let after = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as f64;

        assert!(now >= before);
        assert!(now <= after);
    }

    #[test]
    fn test_date_parse_iso8601() {
        let ts = DateConstructor::parse("2021-01-01T00:00:00.000Z");
        assert_eq!(ts, 1609459200000.0);
    }

    #[test]
    fn test_date_parse_date_only() {
        let ts = DateConstructor::parse("2021-01-01");
        assert_eq!(ts, 1609459200000.0);
    }

    #[test]
    fn test_date_parse_with_milliseconds() {
        let ts = DateConstructor::parse("2021-01-01T12:30:45.123Z");
        let date = JsDate::from_timestamp(ts);
        assert_eq!(date.get_utc_hours(), 12.0);
        assert_eq!(date.get_utc_minutes(), 30.0);
        assert_eq!(date.get_utc_seconds(), 45.0);
        assert_eq!(date.get_utc_milliseconds(), 123.0);
    }

    #[test]
    fn test_date_parse_with_offset() {
        // 2021-01-01T12:00:00+05:30 should be 2021-01-01T06:30:00Z
        let ts = DateConstructor::parse("2021-01-01T12:00:00+05:30");
        let date = JsDate::from_timestamp(ts);
        assert_eq!(date.get_utc_hours(), 6.0);
        assert_eq!(date.get_utc_minutes(), 30.0);
    }

    #[test]
    fn test_date_parse_invalid() {
        let ts = DateConstructor::parse("invalid");
        assert!(ts.is_nan());
    }

    #[test]
    fn test_date_utc() {
        // Date.UTC(2021, 0, 1) = 2021-01-01T00:00:00Z
        let ts = DateConstructor::utc(2021, 0, Some(1), None, None, None, None);
        assert_eq!(ts, 1609459200000.0);
    }

    #[test]
    fn test_date_utc_with_time() {
        let ts = DateConstructor::utc(2021, 0, Some(1), Some(12), Some(30), Some(45), Some(123));
        let date = JsDate::from_timestamp(ts);
        assert_eq!(date.get_utc_full_year(), 2021.0);
        assert_eq!(date.get_utc_month(), 0.0);
        assert_eq!(date.get_utc_date(), 1.0);
        assert_eq!(date.get_utc_hours(), 12.0);
        assert_eq!(date.get_utc_minutes(), 30.0);
        assert_eq!(date.get_utc_seconds(), 45.0);
        assert_eq!(date.get_utc_milliseconds(), 123.0);
    }

    #[test]
    fn test_date_utc_two_digit_year() {
        let ts = DateConstructor::utc(99, 0, Some(1), None, None, None, None);
        let date = JsDate::from_timestamp(ts);
        assert_eq!(date.get_utc_full_year(), 1999.0);
    }

    // ===== Getter Tests (Local) =====

    #[test]
    fn test_get_day_of_week() {
        // 2024-01-01 was a Monday
        let ts = DateConstructor::utc(2024, 0, Some(1), None, None, None, None);
        let date = JsDate::from_timestamp(ts);
        assert_eq!(date.get_utc_day(), 1.0); // Monday = 1 in JS
    }

    #[test]
    fn test_get_day_sunday() {
        // 2024-01-07 was a Sunday
        let ts = DateConstructor::utc(2024, 0, Some(7), None, None, None, None);
        let date = JsDate::from_timestamp(ts);
        assert_eq!(date.get_utc_day(), 0.0); // Sunday = 0 in JS
    }

    #[test]
    fn test_get_milliseconds() {
        let date = JsDate::from_timestamp(1609459200123.0);
        assert_eq!(date.get_milliseconds(), 123.0);
    }

    #[test]
    fn test_timezone_offset_type() {
        let date = JsDate::new();
        let offset = date.get_timezone_offset();
        // Should be a number (might be positive, negative, or zero)
        assert!(!offset.is_nan());
    }

    // ===== Setter Tests =====

    #[test]
    fn test_set_time() {
        let date = JsDate::new();
        let new_time = 1609459200000.0;
        let result = date.set_time(new_time);
        assert_eq!(result, new_time);
        assert_eq!(date.get_time(), new_time);
    }

    #[test]
    fn test_set_full_year() {
        let date = JsDate::from_components(2024, 5, Some(15), Some(12), Some(30), Some(45), Some(100));
        date.set_full_year(2025, None, None);
        assert_eq!(date.get_full_year(), 2025.0);
        assert_eq!(date.get_month(), 5.0);
        assert_eq!(date.get_date(), 15.0);
    }

    #[test]
    fn test_set_month() {
        let date = JsDate::from_components(2024, 0, Some(15), None, None, None, None);
        date.set_month(11, None); // December
        assert_eq!(date.get_month(), 11.0);
        assert_eq!(date.get_date(), 15.0);
    }

    #[test]
    fn test_set_date() {
        let date = JsDate::from_components(2024, 0, Some(1), None, None, None, None);
        date.set_date(25);
        assert_eq!(date.get_date(), 25.0);
    }

    #[test]
    fn test_set_hours() {
        let date = JsDate::from_components(2024, 0, Some(1), Some(0), Some(0), Some(0), Some(0));
        date.set_hours(15, None, None, None);
        assert_eq!(date.get_hours(), 15.0);
    }

    #[test]
    fn test_set_minutes() {
        let date = JsDate::from_components(2024, 0, Some(1), Some(12), Some(0), Some(0), Some(0));
        date.set_minutes(45, None, None);
        assert_eq!(date.get_minutes(), 45.0);
    }

    #[test]
    fn test_set_seconds() {
        let date = JsDate::from_components(2024, 0, Some(1), Some(12), Some(30), Some(0), Some(0));
        date.set_seconds(59, None);
        assert_eq!(date.get_seconds(), 59.0);
    }

    #[test]
    fn test_set_milliseconds() {
        let date = JsDate::from_components(2024, 0, Some(1), Some(12), Some(30), Some(45), Some(0));
        date.set_milliseconds(999);
        assert_eq!(date.get_milliseconds(), 999.0);
    }

    // ===== UTC Setter Tests =====

    #[test]
    fn test_set_utc_full_year() {
        let date = JsDate::from_timestamp(DateConstructor::utc(2024, 0, Some(1), None, None, None, None));
        date.set_utc_full_year(2030, None, None);
        assert_eq!(date.get_utc_full_year(), 2030.0);
    }

    #[test]
    fn test_set_utc_month() {
        let date = JsDate::from_timestamp(DateConstructor::utc(2024, 0, Some(15), None, None, None, None));
        date.set_utc_month(6, None); // July
        assert_eq!(date.get_utc_month(), 6.0);
    }

    #[test]
    fn test_set_utc_date() {
        let date = JsDate::from_timestamp(DateConstructor::utc(2024, 0, Some(1), None, None, None, None));
        date.set_utc_date(20);
        assert_eq!(date.get_utc_date(), 20.0);
    }

    #[test]
    fn test_set_utc_hours() {
        let date = JsDate::from_timestamp(DateConstructor::utc(2024, 0, Some(1), Some(0), None, None, None));
        date.set_utc_hours(23, None, None, None);
        assert_eq!(date.get_utc_hours(), 23.0);
    }

    // ===== Formatting Tests =====

    #[test]
    fn test_to_iso_string() {
        let date = JsDate::from_timestamp(1609459200000.0); // 2021-01-01T00:00:00.000Z
        let iso = date.to_iso_string().unwrap();
        assert_eq!(iso, "2021-01-01T00:00:00.000Z");
    }

    #[test]
    fn test_to_iso_string_with_milliseconds() {
        let date = JsDate::from_timestamp(1609459200123.0);
        let iso = date.to_iso_string().unwrap();
        assert_eq!(iso, "2021-01-01T00:00:00.123Z");
    }

    #[test]
    fn test_to_iso_string_invalid() {
        let date = JsDate::invalid();
        let result = date.to_iso_string();
        assert!(result.is_err());
    }

    #[test]
    fn test_to_json() {
        let date = JsDate::from_timestamp(1609459200000.0);
        let json = date.to_json();
        assert_eq!(json, Some("2021-01-01T00:00:00.000Z".to_string()));
    }

    #[test]
    fn test_to_json_invalid() {
        let date = JsDate::invalid();
        let json = date.to_json();
        assert_eq!(json, None);
    }

    #[test]
    fn test_to_string_format() {
        let date = JsDate::from_timestamp(1609459200000.0);
        let s = date.to_string();
        // Should contain day name, month name, date, year, time, and timezone
        assert!(s.contains("2021"));
        assert!(s.contains("Jan"));
        assert!(s.contains("GMT"));
    }

    #[test]
    fn test_to_string_invalid() {
        let date = JsDate::invalid();
        assert_eq!(date.to_string(), "Invalid Date");
    }

    #[test]
    fn test_to_date_string() {
        let date = JsDate::from_timestamp(1609459200000.0);
        let s = date.to_date_string();
        assert!(s.contains("2021"));
        assert!(s.contains("Jan"));
        assert!(!s.contains(":")); // Should not contain time
    }

    #[test]
    fn test_to_time_string() {
        let date = JsDate::from_timestamp(1609459200000.0);
        let s = date.to_time_string();
        assert!(s.contains(":")); // Should contain time separator
        assert!(s.contains("GMT"));
    }

    #[test]
    fn test_to_locale_string() {
        let date = JsDate::from_timestamp(1609459200000.0);
        let s = date.to_locale_string();
        assert!(s.contains("2021"));
        assert!(s.contains(":"));
    }

    #[test]
    fn test_to_locale_date_string() {
        let date = JsDate::from_timestamp(1609459200000.0);
        let s = date.to_locale_date_string();
        assert!(s.contains("2021"));
        assert!(!s.contains(":")); // Should not contain time
    }

    #[test]
    fn test_to_locale_time_string() {
        let date = JsDate::from_timestamp(1609459200000.0);
        let s = date.to_locale_time_string();
        assert!(s.contains(":"));
    }

    // ===== Edge Cases =====

    #[test]
    fn test_value_of() {
        let date = JsDate::from_timestamp(1609459200000.0);
        assert_eq!(date.value_of(), 1609459200000.0);
    }

    #[test]
    fn test_default() {
        let date = JsDate::default();
        assert!(date.is_valid());
    }

    #[test]
    fn test_epoch() {
        let date = JsDate::from_timestamp(0.0);
        assert_eq!(date.get_utc_full_year(), 1970.0);
        assert_eq!(date.get_utc_month(), 0.0);
        assert_eq!(date.get_utc_date(), 1.0);
        assert_eq!(date.get_utc_hours(), 0.0);
        assert_eq!(date.get_utc_minutes(), 0.0);
        assert_eq!(date.get_utc_seconds(), 0.0);
        assert_eq!(date.get_utc_milliseconds(), 0.0);
    }

    #[test]
    fn test_negative_timestamp() {
        // Before Unix epoch
        let date = JsDate::from_timestamp(-86400000.0); // One day before epoch
        assert_eq!(date.get_utc_full_year(), 1969.0);
        assert_eq!(date.get_utc_month(), 11.0); // December
        assert_eq!(date.get_utc_date(), 31.0);
    }

    #[test]
    fn test_far_future_date() {
        // Year 3000
        let ts = DateConstructor::utc(3000, 0, Some(1), None, None, None, None);
        let date = JsDate::from_timestamp(ts);
        assert_eq!(date.get_utc_full_year(), 3000.0);
    }

    #[test]
    fn test_parse_mm_dd_yyyy() {
        let ts = DateConstructor::parse("12/25/2024");
        let date = JsDate::from_timestamp(ts);
        assert!(date.is_valid());
        assert_eq!(date.get_utc_month(), 11.0); // December = 11
        assert_eq!(date.get_utc_date(), 25.0);
        assert_eq!(date.get_utc_full_year(), 2024.0);
    }

    #[test]
    fn test_clone() {
        let date1 = JsDate::from_timestamp(1609459200000.0);
        let date2 = date1.clone();

        // Both should point to the same internal data
        assert_eq!(date1.get_time(), date2.get_time());

        // Modifying one should affect the other (shared Rc)
        date1.set_time(0.0);
        assert_eq!(date2.get_time(), 0.0);
    }

    #[test]
    fn test_leap_year() {
        // Feb 29, 2024 (leap year)
        let date = JsDate::from_components(2024, 1, Some(29), None, None, None, None);
        assert!(date.is_valid());
        assert_eq!(date.get_date(), 29.0);
    }

    #[test]
    fn test_invalid_leap_year() {
        // Feb 29, 2023 (not a leap year) - should create invalid date
        let result = NaiveDate::from_ymd_opt(2023, 2, 29);
        assert!(result.is_none()); // This confirms the date is invalid
    }

    #[test]
    fn test_all_months() {
        for month in 0..12 {
            let date = JsDate::from_components(2024, month, Some(1), None, None, None, None);
            assert!(date.is_valid(), "Month {} should be valid", month);
            assert_eq!(date.get_month(), month as f64);
        }
    }

    #[test]
    fn test_utc_vs_local_getters() {
        // Create a date at a known UTC time
        let date = JsDate::from_timestamp(1609459200000.0); // 2021-01-01T00:00:00Z

        // UTC getters should return the exact UTC values
        assert_eq!(date.get_utc_full_year(), 2021.0);
        assert_eq!(date.get_utc_month(), 0.0);
        assert_eq!(date.get_utc_date(), 1.0);
        assert_eq!(date.get_utc_hours(), 0.0);

        // Local getters may differ based on timezone
        // We can't predict exact values, but they should be numbers
        assert!(!date.get_full_year().is_nan());
        assert!(!date.get_month().is_nan());
        assert!(!date.get_date().is_nan());
        assert!(!date.get_hours().is_nan());
    }
}

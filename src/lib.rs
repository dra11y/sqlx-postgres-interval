use std::mem;

use serde::{Deserialize, Serialize};
use sqlx::{
    Decode, Encode, Postgres, Type,
    encode::IsNull,
    error::BoxDynError,
    postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef, types::PgInterval},
};

/// A type that mimics [`sqlx::postgres::types::PgInterval`] but provides
/// both [`serde::Serialize`] and [`serde::Deserialize`]
/// into and from ISO 8601 string format.
///
/// ISO 8601 Duration Format:
/// `P(n)Y(n)M(n)DT(n)H(n)M(n)S`
/// Where:
/// P - "period"/duration designator (always present at beginning)
/// (n) - integer
/// Y - follows number of years
/// M - follows number of months
/// W - follows number of weeks
/// D - follows number of days
/// T - designator that precedes time components
/// H - follows number of hours
/// M - follows number of minutes
/// S - follows number of seconds
///
/// See also:
///   - https://en.wikipedia.org/wiki/ISO_8601#Durations
///   - https://www.digi.com/resources/documentation/digidocs/90001488-13/reference/r_iso_8601_duration_format.htm

#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interval {
    pub months: i32,
    pub days: i32,
    pub microseconds: i64,
}

impl Serialize for Interval {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let Self {
            months,
            days,
            microseconds,
        } = self.clone();
        let pgi = pg_interval::Interval {
            months,
            days,
            microseconds,
        };
        serializer.serialize_str(&pgi.to_iso_8601())
    }
}

impl<'de> Deserialize<'de> for Interval {
    fn deserialize<D>(deserializer: D) -> Result<Interval, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let pgi = pg_interval::Interval::from_iso(&s).map_err(|error| {
            serde::de::Error::custom(match error {
                pg_interval::ParseError::ParseIntErr(parse_int_error) => {
                    parse_int_error.to_string()
                }
                pg_interval::ParseError::ParseFloatErr(parse_float_error) => {
                    parse_float_error.to_string()
                }
                pg_interval::ParseError::InvalidYearMonth(invalid_year_month) => invalid_year_month,
                pg_interval::ParseError::InvalidTime(invalid_time) => invalid_time,
                pg_interval::ParseError::InvalidInterval(invalid_interval) => invalid_interval,
            })
        })?;
        Ok(Interval {
            months: pgi.months,
            days: pgi.days,
            microseconds: pgi.microseconds,
        })
    }
}

impl Type<Postgres> for Interval {
    fn type_info() -> PgTypeInfo {
        PgInterval::type_info()
    }
}

impl PgHasArrayType for Interval {
    fn array_type_info() -> PgTypeInfo {
        PgInterval::array_type_info()
    }
}

impl<'de> Decode<'de, Postgres> for Interval {
    fn decode(value: PgValueRef<'de>) -> Result<Self, BoxDynError> {
        let PgInterval {
            months,
            days,
            microseconds,
        } = PgInterval::decode(value)?;
        Ok(Interval {
            months,
            days,
            microseconds,
        })
    }
}

impl Encode<'_, Postgres> for Interval {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        let Self {
            months,
            days,
            microseconds,
        } = self.clone();
        let pg_interval = PgInterval {
            months,
            days,
            microseconds,
        };
        pg_interval.encode_by_ref(buf)
    }

    fn size_hint(&self) -> usize {
        2 * mem::size_of::<i64>()
    }
}

impl TryFrom<std::time::Duration> for Interval {
    type Error = BoxDynError;

    /// Convert a `std::time::Duration` to a `PgInterval`
    ///
    /// This returns an error if there is a loss of precision using nanoseconds or if there is a
    /// microsecond overflow.
    fn try_from(value: std::time::Duration) -> Result<Self, BoxDynError> {
        if value.as_nanos() % 1000 != 0 {
            return Err("PostgreSQL `INTERVAL` does not support nanoseconds precision".into());
        }

        Ok(Self {
            months: 0,
            days: 0,
            microseconds: value.as_micros().try_into()?,
        })
    }
}

#[cfg(feature = "chrono")]
impl TryFrom<chrono::Duration> for Interval {
    type Error = BoxDynError;

    /// Convert a `chrono::Duration` to an `Interval`.
    ///
    /// This returns an error if there is a loss of precision using nanoseconds or if there is a
    /// nanosecond overflow.
    fn try_from(value: chrono::Duration) -> Result<Self, BoxDynError> {
        value
            .num_nanoseconds()
            .map_or::<Result<_, Self::Error>, _>(
                Err("Overflow has occurred for PostgreSQL `INTERVAL`".into()),
                |nanoseconds| {
                    if nanoseconds % 1000 != 0 {
                        return Err(
                            "PostgreSQL `INTERVAL` does not support nanoseconds precision".into(),
                        );
                    }
                    Ok(())
                },
            )?;

        value.num_microseconds().map_or(
            Err("Overflow has occurred for PostgreSQL `INTERVAL`".into()),
            |microseconds| {
                Ok(Self {
                    months: 0,
                    days: 0,
                    microseconds,
                })
            },
        )
    }
}

#[cfg(feature = "time")]
impl TryFrom<time::Duration> for Interval {
    type Error = BoxDynError;

    /// Convert a `time::Duration` to a `PgInterval`.
    ///
    /// This returns an error if there is a loss of precision using nanoseconds or if there is a
    /// microsecond overflow.
    fn try_from(value: time::Duration) -> Result<Self, BoxDynError> {
        if value.whole_nanoseconds() % 1000 != 0 {
            return Err("PostgreSQL `INTERVAL` does not support nanoseconds precision".into());
        }

        Ok(Self {
            months: 0,
            days: 0,
            microseconds: value.whole_microseconds().try_into()?,
        })
    }
}

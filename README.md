# sqlx-postgres-interval

The "current" (2024-12-18) version of [`sqlx`](https://crates.io/crates/sqlx) ([GitHub](https://github.com/launchbadge/sqlx)) for Postgres (0.8.2) does not derive `serde::Serialize` or `serde::Deserialize` for its type, `sqlx::postgres::types::PgInterval`, that represents the Postgres [INTERVAL](https://www.postgresql.org/docs/current/datatype-datetime.html#DATATYPE-INTERVAL-INPUT) type.

## Usage

Just add this crate and use its type:

```rs
use serde::{Deserialize, Serialize};
#[derive(sqlx::FromRow, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Video {
    pub id: i32,
    pub title: String,
    // ...
    pub duration: Option<sqlx_postgres_interval::Interval>,
}
```

## Features
The `chrono` and `time` **features** will convert to/from their respective `Duration`s. I haven't fully tested this; the code is copied verbatim from the current `sqlx::postgres::types::PgInterval` implementations.

## Motivation

My database has a couple `INTERVAL` fields, and I don't want to have to manually implement these in my project, therefore, this crate now exists. Hopefully it will be obsoleted if `serde::Serialize` and `serde::Deserialize` get implemented for it (check at https://github.com/launchbadge/sqlx/blob/main/sqlx-postgres/src/types/interval.rs).

Unfortunately, one cannot implement `serde` (or any other external traits) for another external crate, hence the need to wrap the values.

## Implementation

This crate wraps `sqlx::postgres::types::PgInterval` in its `sqlx::Decode` and `sqlx::Encode` implementations, and uses the `pg_interval` crate to Serialize/Deserialize it to/fron `String`.

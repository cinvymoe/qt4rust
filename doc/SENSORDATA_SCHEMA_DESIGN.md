# SensorData Database Schema Consistency Design

**Date**: 2025-04-15  
**Author**: Sisyphus Assistant  
**Status**: Draft  
**Components**: sensor-core, qt-rust-demo repositories

## Problem Statement

Currently when adding new fields to `SensorData` struct, it is easy to forget to sync the SQLite database schema, resulting in data loss. For example:

- `SensorData` contains `digital_input_0` and `digital_input_1` switch fields
- But database `sensor_data` table only stores three AD fields (ad1_load, ad2_radius, ad3_angle)
- Switch data is completely discarded

We need a mechanism to catch such inconsistencies at **compile time** or **test phase**.

## Goals

1. **Compile-time Protection**: Adding/removing fields must sync database code, otherwise compilation fails
2. **Automation**: Reduce manual SQL maintenance workload and errors
3. **Verifiable**: Provide tests to validate struct-to-database consistency
4. **Backward Compatible**: Smooth migration without breaking existing functionality

## Design Overview

Using **dual protection** strategy:

```
┌─────────────────────────────────────────────────────────────┐
│                    Dual Protection System                   │
├─────────────────────────────────────────────────────────────┤
│  Layer 1: Compile-time Check (Primary)                      │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ DatabaseSchema Trait System                          │  │
│  │ • Force implement columns() and field_values()      │  │
│  │ • New fields must update both, or compile fails     │  │
│  │ • Auto-generate SQL statements from trait           │  │
│  └──────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│  Layer 2: Runtime Validation (Fallback)                     │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ SchemaVerifier Test Guard                            │  │
│  │ • Runtime compare trait vs actual DB schema         │  │
│  │ • CI/CD auto-execution to catch mismatches          │  │
│  │ • Clear error reporting                              │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Layer 1: DatabaseSchema Trait System

### Core Trait Definition

Add `src/storage/schema.rs` in sensor-core crate:

```rust
use rusqlite::types::ToSql;

/// Database column definition
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColumnDef {
    /// Field name (snake_case, matches SQL)
    pub name: &'static str,
    /// SQL type (e.g., "REAL", "BOOLEAN", "INTEGER")
    pub sql_type: &'static str,
    /// Whether NULL is allowed
    pub nullable: bool,
}

/// Database Schema Definition Trait
/// 
/// Types implementing this trait must provide complete table schema info
/// for auto-generating SQL statements and parameter binding.
pub trait DatabaseSchema {
    /// Returns table name
    fn table_name() -> &'static str;
    
    /// Returns all column definitions (must include all fields)
    fn columns() -> &'static [ColumnDef];
    
    /// Generates CREATE TABLE SQL statement
    fn create_table_sql() -> String;
    
    /// Returns column names for INSERT (excludes id and created_at)
    fn insert_columns() -> &'static [&'static str];
    
    /// Generates INSERT statement template
    fn insert_sql() -> String;
    
    /// Gets field values for parameter binding
    /// 
    /// # Safety
    /// Returned Vec length must match columns() count exactly
    fn field_values(&self) -> Vec<&dyn ToSql>;
}
```

### SensorData Implementation Example

```rust
// crates/sensor-core/src/data/sensor_data.rs

use crate::storage::{ColumnDef, DatabaseSchema};
use rusqlite::types::ToSql;

#[derive(Debug, Clone, PartialEq)]
pub struct SensorData {
    pub ad1_load: f64,
    pub ad2_radius: f64,
    pub ad3_angle: f64,
    pub digital_input_0: bool,
    pub digital_input_1: bool,
}

impl DatabaseSchema for SensorData {
    fn table_name() -> &'static str {
        "sensor_data"
    }
    
    fn columns() -> &'static [ColumnDef] {
        &[
            ColumnDef { 
                name: "ad1_load", 
                sql_type: "REAL", 
                nullable: false 
            },
            ColumnDef { 
                name: "ad2_radius", 
                sql_type: "REAL", 
                nullable: false 
            },
            ColumnDef { 
                name: "ad3_angle", 
                sql_type: "REAL", 
                nullable: false 
            },
            ColumnDef { 
                name: "digital_input_0", 
                sql_type: "BOOLEAN", 
                nullable: false 
            },
            ColumnDef { 
                name: "digital_input_1", 
                sql_type: "BOOLEAN", 
                nullable: false 
            },
        ]
    }
    
    fn field_values(&self) -> Vec<&dyn ToSql> {
        vec![
            &self.ad1_load,
            &self.ad2_radius,
            &self.ad3_angle,
            &self.digital_input_0,
            &self.digital_input_1,
        ]
    }
}
```

### Protection Mechanism

**Scenario 1: Adding a field**

If you add `ad4_temperature` field but forget to update `field_values()`:

```rust
fn field_values(&self) -> Vec<&dyn ToSql> {
    vec![
        &self.ad1_load,
        &self.ad2_radius,
        &self.ad3_angle,
        &self.digital_input_0,
        &self.digital_input_1,
        // Missing &self.ad4_temperature!
    ]
}
```

**Result**: Runtime assertion will fail because `columns()` returns 6 columns but `field_values()` only returns 5 values.

**Scenario 2: Removing a field**

If you remove a field but forget to update `field_values()`:

```rust
fn field_values(&self) -> Vec<&dyn ToSql> {
    vec![
        &self.ad1_load,
        &self.digital_input_1,  // Compile error: field doesn't exist
    ]
}
```

**Result**: Compile-time error because the field no longer exists in the struct.

## Layer 2: SchemaVerifier Test Guard

### Test Implementation

Create `tests/schema_verification.rs`:

```rust
use qt_rust_demo::repositories::SqliteStorageRepository;
use sensor_core::DatabaseSchema;
use std::collections::HashSet;

#[derive(Debug)]
pub enum SchemaMismatchError {
    DatabaseError(String),
    ColumnMismatch {
        table: String,
        missing_in_db: Vec<String>,
        extra_in_db: Vec<String>,
    },
}

/// Verify struct definition matches actual database schema
pub async fn verify_schema<T: DatabaseSchema>(
    repo: &SqliteStorageRepository,
) -> Result<(), SchemaMismatchError> {
    // Get all fields from trait definition
    let trait_fields: HashSet<String> = T::columns()
        .iter()
        .map(|c| c.name.to_string())
        .collect();
    
    // Get actual table schema from database
    let db_columns = repo
        .get_table_columns(T::table_name())
        .await
        .map_err(|e| SchemaMismatchError::DatabaseError(e))?;
    
    let db_fields: HashSet<String> = db_columns
        .into_iter()
        .filter(|col| col != "id" && col != "created_at")
        .collect();
    
    // Check differences
    let missing_in_db: Vec<_> = trait_fields.difference(&db_fields).cloned().collect();
    let extra_in_db: Vec<_> = db_fields.difference(&trait_fields).cloned().collect();
    
    if !missing_in_db.is_empty() || !extra_in_db.is_empty() {
        return Err(SchemaMismatchError::ColumnMismatch {
            table: T::table_name().to_string(),
            missing_in_db,
            extra_in_db,
        });
    }
    
    Ok(())
}

#[tokio::test]
async fn test_sensor_data_schema_matches_database() {
    let repo = SqliteStorageRepository::new(":memory:").await.unwrap();
    
    match verify_schema::<SensorData>(&repo).await {
        Ok(()) => {
            // Schema is consistent
        }
        Err(SchemaMismatchError::ColumnMismatch { 
            table, 
            missing_in_db, 
            extra_in_db 
        }) => {
            let mut error_msg = format!("Schema mismatch in table '{}':\n", table);
            
            if !missing_in_db.is_empty() {
                error_msg.push_str(&format!(
                    "  Fields in SensorData but missing in DB: {:?}\n",
                    missing_in_db
                ));
                error_msg.push_str("  Action: Update database migration to add these columns\n");
            }
            
            if !extra_in_db.is_empty() {
                error_msg.push_str(&format!(
                    "  Fields in DB but missing in SensorData: {:?}\n",
                    extra_in_db
                ));
                error_msg.push_str("  Action: Update SensorData struct or remove from DB\n");
            }
            
            panic!("{}", error_msg);
        }
        Err(SchemaMismatchError::DatabaseError(e)) => {
            panic!("Database error during schema verification: {}", e);
        }
    }
}
```

## Implementation Plan

### Phase 1: Add DatabaseSchema Trait (sensor-core)

1. Create `crates/sensor-core/src/storage/schema.rs` with trait definition
2. Implement default methods for `create_table_sql()` and `insert_sql()`
3. Add module export in `crates/sensor-core/src/lib.rs`

### Phase 2: Implement for SensorData

1. Implement `DatabaseSchema` for `SensorData` in `sensor_data.rs`
2. Define all 5 columns with correct SQL types
3. Implement `field_values()` to return all field references

### Phase 3: Update SQLite Repository

1. Modify `init_tables()` to use `SensorData::create_table_sql()`
2. Modify `save_sensor_data_batch()` to use trait methods
3. Add runtime assertion checking field count match
4. Add helper method `get_table_columns()` for testing

### Phase 4: Add Test Guard

1. Create `tests/schema_verification.rs`
2. Implement `SchemaVerifier` with detailed error reporting
3. Add test to CI/CD pipeline

### Phase 5: Database Migration

Since current database only has 3 columns but SensorData has 5 fields:

```sql
-- Migration script
ALTER TABLE sensor_data ADD COLUMN digital_input_0 BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE sensor_data ADD COLUMN digital_input_1 BOOLEAN NOT NULL DEFAULT 0;
```

## Benefits

1. **Compile-time Safety**: Adding fields requires updating both struct and trait, or it won't compile
2. **Auto-generated SQL**: No manual SQL writing, reducing typos
3. **Test Coverage**: CI/CD automatically catches any schema mismatches
4. **Clear Errors**: When mismatch occurs, error messages clearly indicate what's wrong and how to fix

## Migration Notes

- Existing code continues to work during migration
- New trait methods have default implementations
- Database migration can be done independently of code changes
- Tests serve as safety net for future changes

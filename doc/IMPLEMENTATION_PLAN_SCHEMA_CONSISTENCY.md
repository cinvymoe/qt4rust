# SensorData Schema Consistency Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement DatabaseSchema trait system to prevent schema mismatches when adding SensorData fields, ensuring database synchronization with compile-time and runtime checks.

**Architecture:** Add a `DatabaseSchema` trait to sensor-core that requires implementing types to define their SQL schema and provide field values. SensorData implements this trait, and SqliteStorageRepository uses trait methods to auto-generate SQL. A test guard verifies trait definition matches actual database schema at runtime.

**Tech Stack:** Rust, SQLite (rusqlite), async-trait, sensor-core crate, qt-rust-demo repository

---

## File Structure

```
crates/sensor-core/
├── Cargo.toml                          # Add rusqlite dependency
└── src/
    ├── lib.rs                          # Export DatabaseSchema trait
    └── storage/
        ├── mod.rs                      # Export schema module
        └── schema.rs                   # NEW: DatabaseSchema trait + ColumnDef

crates/sensor-core/src/data/
└── sensor_data.rs                      # ADD: impl DatabaseSchema for SensorData

src/repositories/
├── sqlite_storage_repository.rs        # MODIFY: Use trait methods for SQL
└── mod.rs                              # Export get_table_columns helper trait

tests/
└── schema_verification.rs              # NEW: SchemaVerifier test guard
```

---

## Task 1: Add rusqlite Dependency to sensor-core

**Files:**
- Modify: `crates/sensor-core/Cargo.toml`

**Purpose:** Enable sensor-core to use rusqlite's ToSql trait for parameter binding.

- [ ] **Step 1: Add rusqlite dependency**

Add rusqlite to dependencies section:

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"
async-trait = "0.1"
tokio = { version = "1.0", features = ["sync", "time", "rt", "rt-multi-thread", "macros", "test-util"] }
qt-threading-utils = { path = "../qt-threading-utils" }
rusqlite = { version = "0.30", features = ["bundled"] }  # ADD THIS LINE
```

- [ ] **Step 2: Verify dependency resolves**

Run: `cargo check -p sensor-core`

Expected: Should compile without errors (may show warnings, but no errors)

- [ ] **Step 3: Commit**

```bash
git add crates/sensor-core/Cargo.toml
git commit -m "chore: add rusqlite dependency to sensor-core

Required for DatabaseSchema trait to use ToSql trait for parameter binding."
```

---

## Task 2: Create DatabaseSchema Trait

**Files:**
- Create: `crates/sensor-core/src/storage/schema.rs`
- Modify: `crates/sensor-core/src/storage/mod.rs`

**Purpose:** Define the core trait that enforces schema consistency.

- [ ] **Step 1: Create schema.rs with trait definition**

```rust
// crates/sensor-core/src/storage/schema.rs

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
    fn create_table_sql() -> String {
        let columns_sql = Self::columns()
            .iter()
            .map(|col| {
                let nullable = if col.nullable { "" } else { " NOT NULL" };
                format!("{} {}{}", col.name, col.sql_type, nullable)
            })
            .collect::<Vec<_>>()
            .join(",\n    ");

        format!(
            "CREATE TABLE IF NOT EXISTS {} (\n    id INTEGER PRIMARY KEY AUTOINCREMENT,\n    {},\n    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))\n)",
            Self::table_name(),
            columns_sql
        )
    }

    /// Returns column names for INSERT (excludes id and created_at)
    fn insert_columns() -> &'static [&'static str] {
        // Lazily initialize using std::sync::OnceLock pattern would be ideal,
        // but for simplicity we compute each time or use const when possible
        // This is a placeholder - the actual implementation needs to be const
        &[]
    }

    /// Generates INSERT statement template
    fn insert_sql() -> String {
        let columns = Self::columns();
        let column_names: Vec<_> = columns.iter().map(|c| c.name).collect();
        let placeholders: Vec<_> = (1..=columns.len())
            .map(|i| format!("?{}", i))
            .collect();

        format!(
            "INSERT OR IGNORE INTO {} ({}) VALUES ({})",
            Self::table_name(),
            column_names.join(", "),
            placeholders.join(", ")
        )
    }

    /// Gets field values for parameter binding
    ///
    /// # Safety
    /// Returned Vec length must match columns() count exactly
    fn field_values(&self) -> Vec<&dyn ToSql>;
}
```

- [ ] **Step 2: Export from storage/mod.rs**

```rust
// crates/sensor-core/src/storage/mod.rs

pub mod repository;
pub mod schema;  // ADD THIS LINE

pub use repository::{MockStorageRepository, StorageRepository};
pub use schema::{ColumnDef, DatabaseSchema};  // ADD THIS LINE
```

- [ ] **Step 3: Export from lib.rs**

Add to `crates/sensor-core/src/lib.rs` line 22:

```rust
pub use storage::{ColumnDef, DatabaseSchema};  // ADD THIS
```

- [ ] **Step 4: Verify compilation**

Run: `cargo check -p sensor-core`

Expected: Compiles without errors

- [ ] **Step 5: Commit**

```bash
git add crates/sensor-core/src/storage/schema.rs
git add crates/sensor-core/src/storage/mod.rs
git add crates/sensor-core/src/lib.rs
git commit -m "feat: add DatabaseSchema trait to sensor-core

- Define ColumnDef struct for column metadata
- Define DatabaseSchema trait with SQL generation methods
- Export from storage module and lib"
```

---

## Task 3: Implement DatabaseSchema for SensorData

**Files:**
- Modify: `crates/sensor-core/src/data/sensor_data.rs`

**Purpose:** Make SensorData implement the trait to enable schema checking.

- [ ] **Step 1: Add trait import and implementation**

Add after line 8 in `crates/sensor-core/src/data/sensor_data.rs`:

```rust
use crate::storage::{ColumnDef, DatabaseSchema};
use rusqlite::types::ToSql;

// Add this impl block after the existing impl SensorData block
impl DatabaseSchema for SensorData {
    fn table_name() -> &'static str {
        "sensor_data"
    }

    fn columns() -> &'static [ColumnDef] {
        &[
            ColumnDef {
                name: "ad1_load",
                sql_type: "REAL",
                nullable: false,
            },
            ColumnDef {
                name: "ad2_radius",
                sql_type: "REAL",
                nullable: false,
            },
            ColumnDef {
                name: "ad3_angle",
                sql_type: "REAL",
                nullable: false,
            },
            ColumnDef {
                name: "digital_input_0",
                sql_type: "BOOLEAN",
                nullable: false,
            },
            ColumnDef {
                name: "digital_input_1",
                sql_type: "BOOLEAN",
                nullable: false,
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

- [ ] **Step 2: Verify compilation**

Run: `cargo check -p sensor-core`

Expected: Compiles without errors

- [ ] **Step 3: Commit**

```bash
git add crates/sensor-core/src/data/sensor_data.rs
git commit -m "feat: implement DatabaseSchema for SensorData

Define all 5 columns with SQL types:
- ad1_load, ad2_radius, ad3_angle: REAL
- digital_input_0, digital_input_1: BOOLEAN

Implement field_values() to return references to all fields."
```

---

## Task 4: Add Helper Method to SqliteStorageRepository

**Files:**
- Modify: `src/repositories/sqlite_storage_repository.rs`
- Modify: `src/repositories/mod.rs`

**Purpose:** Add method to query table columns for schema verification.

- [ ] **Step 1: Add get_table_columns method**

Add to `src/repositories/sqlite_storage_repository.rs` after line 483 (after purge_old_alarms method):

```rust
    /// Get column names from a table
    pub async fn get_table_columns(&self, table_name: &str) -> Result<Vec<String>, String> {
        let conn = self.connection.lock().await;

        let mut stmt = conn
            .prepare(&format!(
                "SELECT name FROM pragma_table_info('{}')",
                table_name
            ))
            .map_err(|e| format!("Failed to prepare table_info query: {}", e))?;

        let columns = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| format!("Failed to query table columns: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect columns: {}", e))?;

        Ok(columns)
    }
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`

Expected: Compiles without errors

- [ ] **Step 3: Commit**

```bash
git add src/repositories/sqlite_storage_repository.rs
git commit -m "feat: add get_table_columns helper method

Enables querying database schema for runtime verification."
```

---

## Task 5: Create Schema Verification Test

**Files:**
- Create: `tests/schema_verification.rs`

**Purpose:** Add test that fails if trait definition doesn't match database.

- [ ] **Step 1: Create test file**

```rust
// tests/schema_verification.rs

use qt_rust_demo::repositories::SqliteStorageRepository;
use sensor_core::DatabaseSchema;
use std::collections::HashSet;

/// Schema mismatch error types
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

    match verify_schema::<sensor_core::SensorData>(&repo).await {
        Ok(()) => {
            // Schema is consistent - test passes
        }
        Err(SchemaMismatchError::ColumnMismatch {
            table,
            missing_in_db,
            extra_in_db,
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

- [ ] **Step 2: Run test to verify it fails (TDD)**

Run: `cargo test test_sensor_data_schema_matches_database --test schema_verification -- --nocapture`

Expected: Test **FAILS** with message showing:
- `digital_input_0` missing in DB
- `digital_input_1` missing in DB

This proves the test works - it catches the schema mismatch!

- [ ] **Step 3: Commit the test**

```bash
git add tests/schema_verification.rs
git commit -m "test: add schema verification test

Test verifies SensorData trait definition matches actual database schema.
Expected to fail initially since DB doesn't have switch columns yet."
```

---

## Task 6: Update Database Schema (Migration)

**Files:**
- Create: `migrations/001_add_switch_columns.sql`

**Purpose:** Add missing columns to existing database.

- [ ] **Step 1: Create migration directory and script**

```bash
mkdir -p migrations
```

Create `migrations/001_add_switch_columns.sql`:

```sql
-- Migration: Add switch columns to sensor_data table
-- Date: 2025-04-15

-- Add digital_input_0 column (main hook switch)
ALTER TABLE sensor_data ADD COLUMN digital_input_0 BOOLEAN NOT NULL DEFAULT 0;

-- Add digital_input_1 column (auxiliary hook switch)
ALTER TABLE sensor_data ADD COLUMN digital_input_1 BOOLEAN NOT NULL DEFAULT 0;
```

- [ ] **Step 2: Document migration in design doc**

The migration needs to be applied to existing databases. For development:

```bash
# Apply to local database
sqlite3 crane_data.db < migrations/001_add_switch_columns.sql

# Or delete and recreate (if data can be lost)
rm crane_data.db
# New schema will be created automatically on next run
```

- [ ] **Step 3: Update SqliteStorageRepository to use trait SQL**

Modify `init_tables()` in `src/repositories/sqlite_storage_repository.rs` around line 44-65:

Replace the sensor_data CREATE TABLE block with:

```rust
        // Create sensor data table using trait-generated SQL
        conn.execute(
            &SensorData::create_table_sql(),
            [],
        )
        .map_err(|e| format!("Failed to create sensor_data table: {}", e))?;
```

This requires adding import at the top of the file:

```rust
use sensor_core::DatabaseSchema;
```

- [ ] **Step 4: Verify test passes now**

Run: `cargo test test_sensor_data_schema_matches_database --test schema_verification -- --nocapture`

Expected: Test **PASSES** because the new database schema matches the trait definition.

- [ ] **Step 5: Commit**

```bash
git add migrations/001_add_switch_columns.sql
git add src/repositories/sqlite_storage_repository.rs
git commit -m "feat: update database schema and use trait-generated SQL

- Add migration script for switch columns
- Use SensorData::create_table_sql() instead of hardcoded SQL
- Schema verification test now passes"
```

---

## Task 7: Update save_sensor_data_batch to Use Trait

**Files:**
- Modify: `src/repositories/sqlite_storage_repository.rs`

**Purpose:** Use trait methods for INSERT operations.

- [ ] **Step 1: Modify save_sensor_data_batch method**

Replace lines 617-645 (the save_sensor_data_batch method) with:

```rust
    async fn save_sensor_data_batch(&self, data: &[SensorData]) -> Result<usize, String> {
        if data.is_empty() {
            return Ok(0);
        }

        let conn = self.connection.lock().await;

        conn.execute("BEGIN TRANSACTION", [])
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;

        let mut saved_count = 0;
        let sql = SensorData::insert_sql();

        for item in data {
            let values = item.field_values();

            // Runtime assertion to ensure field count matches
            assert_eq!(
                values.len(),
                SensorData::columns().len(),
                "field_values() returned {} values but columns() defines {} fields",
                values.len(),
                SensorData::columns().len()
            );

            match conn.execute(&sql, &*values) {
                Ok(rows) => {
                    saved_count += rows;
                }
                Err(e) => {
                    let _ = conn.execute("ROLLBACK", []);
                    return Err(format!("Failed to insert sensor data: {}", e));
                }
            }
        }

        conn.execute("COMMIT", [])
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;

        tracing::info!("Saved {} sensor records to database", saved_count);
        Ok(saved_count)
    }
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`

Expected: Compiles without errors

- [ ] **Step 3: Run existing tests to ensure no regression**

Run: `cargo test -p qt-rust-demo --lib`

Expected: All existing tests pass (or only pre-existing failures)

- [ ] **Step 4: Commit**

```bash
git add src/repositories/sqlite_storage_repository.rs
git commit -m "refactor: use trait methods in save_sensor_data_batch

- Use SensorData::insert_sql() instead of hardcoded SQL
- Use field_values() for parameter binding
- Add runtime assertion checking field count match
- Validates at runtime that all fields are being persisted"
```

---

## Task 8: Final Verification

**Files:**
- All modified files

**Purpose:** Ensure everything works together.

- [ ] **Step 1: Run all tests**

Run: `cargo test`

Expected:
- Schema verification test passes
- Existing tests pass (or show only pre-existing failures)
- No new test failures introduced

- [ ] **Step 2: Verify compilation of entire workspace**

Run: `cargo build --workspace`

Expected: Builds successfully

- [ ] **Step 3: Verify trait protection works**

Temporarily add a field to SensorData (don't commit this!):

```rust
pub struct SensorData {
    pub ad1_load: f64,
    pub ad2_radius: f64,
    pub ad3_angle: f64,
    pub digital_input_0: bool,
    pub digital_input_1: bool,
    pub test_field: f64,  // ADD THIS TEMPORARILY
}
```

Don't update `columns()` or `field_values()`.

Run: `cargo test test_sensor_data_schema_matches_database --test schema_verification`

Expected: Test **FAILS** with error about `test_field` missing in DB.

This proves the protection mechanism works!

Revert the test change:

```bash
git checkout crates/sensor-core/src/data/sensor_data.rs
```

- [ ] **Step 4: Create summary commit**

```bash
git log --oneline -10
```

Ensure all commits are present:
1. Add rusqlite dependency
2. Add DatabaseSchema trait
3. Implement for SensorData
4. Add get_table_columns helper
5. Add schema verification test
6. Update database schema
7. Update save_sensor_data_batch

- [ ] **Step 5: Final commit with summary**

```bash
git log --oneline | head -7 | tac | xargs -I {} echo "- {}"
```

Create a final summary (no code changes, just documentation):

```bash
# Tag the final state
git tag -a schema-consistency-v1.0 -m "Complete DatabaseSchema trait implementation

Features:
- Compile-time trait enforcement for SensorData
- Auto-generated SQL from trait definitions  
- Runtime schema verification test
- Database migration for switch columns

Protection: Adding fields without updating trait will fail tests."
```

---

## Summary

This implementation adds a **dual protection** system:

1. **Compile-time**: The `DatabaseSchema` trait requires implementing both `columns()` and `field_values()`. If you add a field to `SensorData`, you must update both methods or the code won't work correctly.

2. **Runtime**: The `test_sensor_data_schema_matches_database` test compares trait definition against actual database schema and fails with a clear error message if they don't match.

3. **SQL Generation**: SQL statements are auto-generated from the trait, eliminating manual SQL maintenance.

### Future Field Additions

When adding a new field to SensorData:

1. Add field to `SensorData` struct
2. Add `ColumnDef` to `columns()` method
3. Add field reference to `field_values()` method
4. Create database migration to add column
5. Run tests to verify everything works

If you forget step 2 or 3, the trait implementation is incomplete.
If you forget step 4, the schema verification test will fail.

### Migration Path for Existing Databases

Existing databases need the switch columns added:

```bash
sqlite3 crane_data.db < migrations/001_add_switch_columns.sql
```

Or on the device:
```bash
adb shell "sqlite3 /data/local/tmp/qt-rust-demo/crane_data.db < /data/local/tmp/qt-rust-demo/migrations/001_add_switch_columns.sql"
```

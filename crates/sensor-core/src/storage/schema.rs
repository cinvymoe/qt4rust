use rusqlite::types::ToSql;

/// Database column definition
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDef {
    /// Field name (snake_case, matches SQL)
    pub name: String,
    /// SQL type (e.g., "REAL", "BOOLEAN", "INTEGER")
    pub sql_type: String,
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

    /// Generates INSERT statement template
    fn insert_sql() -> String {
        let columns = Self::columns();
        let column_names: Vec<_> = columns.iter().map(|c| c.name.clone()).collect();
        let placeholders: Vec<_> = (1..=columns.len()).map(|i| format!("?{}", i)).collect();

        format!(
            "INSERT OR IGNORE INTO {} ({}) VALUES ({})",
            Self::table_name(),
            column_names.join(", "),
            placeholders.join(", ")
        )
    }

    /// Gets field values for parameter binding
    fn field_values(&self) -> Vec<Box<dyn ToSql>>;
}

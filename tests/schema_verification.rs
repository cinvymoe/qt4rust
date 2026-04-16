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

pub async fn verify_schema<T: DatabaseSchema>(
    repo: &SqliteStorageRepository,
) -> Result<(), SchemaMismatchError> {
    let trait_fields: HashSet<String> = T::columns().iter().map(|c| c.name.to_string()).collect();

    let db_columns = repo
        .get_table_columns(T::table_name())
        .await
        .map_err(|e| SchemaMismatchError::DatabaseError(e))?;

    let db_fields: HashSet<String> = db_columns
        .into_iter()
        .filter(|col| col != "id" && col != "created_at")
        .collect();

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
        Ok(()) => {}
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

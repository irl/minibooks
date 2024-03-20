use sqlx::SqliteExecutor;

pub async fn get_settings_int(e: impl SqliteExecutor<'_>, name: &str) -> Option<i64> {
    let result = sqlx::query!(
        "SELECT intValue FROM settings WHERE name=?", name
    )
        .fetch_one(e)
        .await
        .unwrap();
    result.intValue
}

pub async fn get_settings_str(e: impl SqliteExecutor<'_>, name: &str) -> Option<String> {
    let result = sqlx::query!(
        "SELECT strValue FROM settings WHERE name=?", name
    )
        .fetch_one(e)
        .await
        .unwrap();
    result.strValue
}

pub async fn set_settings_int(e: impl SqliteExecutor<'_>, name: &str, value: i64) {
    sqlx::query!(
        "INSERT OR REPLACE INTO settings (name, intValue) VALUES (?, ?)",
        name, value
    )
        .execute(e)
        .await
        .unwrap();
}

pub async fn set_settings_str(e: impl SqliteExecutor<'_>, name: &str, value: i64) {
    sqlx::query!(
        "INSERT OR REPLACE INTO settings (name, strValue) VALUES (?, ?)",
        name, value
    )
        .execute(e)
        .await
        .unwrap();
}

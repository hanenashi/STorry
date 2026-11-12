use crate::ollama::{ChatMessage, Role};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub struct DatabasePath(pub PathBuf);

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Draft {
    title: String,
    scene: String,
    notes: String,
    base_url: String,
    model: String,
    prompt: String,
    messages: Vec<ChatMessage>,
    context_tokens: usize,
}

impl Draft {
    fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() || self.title.chars().count() > 250 {
            return Err("Give the story a title of 1–250 characters.".into());
        }
        if ![8192, 16384, 32768].contains(&self.context_tokens) {
            return Err("Unsupported story memory size.".into());
        }
        if !self.messages.len().is_multiple_of(2)
            || self.messages.iter().enumerate().any(|(index, m)| {
                matches!(m.role, Role::User) != (index % 2 == 0) || m.content.trim().is_empty()
            })
        {
            return Err("This story contains an incomplete conversation.".into());
        }
        if serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 10 * 1024 * 1024 {
            return Err("This story exceeds the 10 MB save limit.".into());
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Story {
    id: String,
    revision: i64,
    updated_at: i64,
    data: Draft,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    id: String,
    revision: i64,
    title: String,
    updated_at: i64,
    words: usize,
    turns: usize,
    excerpt: String,
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn open(path: &DatabasePath) -> Result<Connection, String> {
    let db = Connection::open(&path.0).map_err(|e| e.to_string())?;
    db.busy_timeout(Duration::from_secs(5))
        .map_err(|e| e.to_string())?;
    db.execute_batch("CREATE TABLE IF NOT EXISTS stories (
        id TEXT PRIMARY KEY, revision INTEGER NOT NULL, updated_at INTEGER NOT NULL, data TEXT NOT NULL
    ); CREATE TABLE IF NOT EXISTS metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL);")
        .map_err(|e| e.to_string())?;
    Ok(db)
}

fn validate_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 64 || !id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
    {
        return Err("Invalid story identifier.".into());
    }
    Ok(())
}

fn read(db: &Connection, id: &str) -> Result<Story, String> {
    validate_id(id)?;
    let record: Option<(i64, i64, String)> = db
        .query_row(
            "SELECT revision, updated_at, data FROM stories WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    let (revision, updated_at, json) =
        record.ok_or("Story no longer exists. Refresh the library.")?;
    Ok(Story {
        id: id.into(),
        revision,
        updated_at,
        data: serde_json::from_str(&json)
            .map_err(|e| format!("Saved story could not be read: {e}"))?,
    })
}

fn insert(db: &Connection, id: &str, data: Draft) -> Result<Story, String> {
    validate_id(id)?;
    data.validate()?;
    let updated_at = now();
    db.execute(
        "INSERT INTO stories (id, revision, updated_at, data) VALUES (?1, 1, ?2, ?3)",
        params![
            id,
            updated_at,
            serde_json::to_string(&data).map_err(|e| e.to_string())?
        ],
    )
    .map_err(|e| format!("Could not create story: {e}"))?;
    Ok(Story {
        id: id.into(),
        revision: 1,
        updated_at,
        data,
    })
}

fn update(db: &Connection, id: &str, revision: i64, data: Draft) -> Result<Story, String> {
    validate_id(id)?;
    data.validate()?;
    let updated_at = now();
    let affected = db.execute("UPDATE stories SET revision = revision + 1, updated_at = ?1, data = ?2 WHERE id = ?3 AND revision = ?4",
        params![updated_at, serde_json::to_string(&data).map_err(|e| e.to_string())?, id, revision])
        .map_err(|e| format!("Could not save story: {e}"))?;
    if affected != 1 {
        return Err("This story changed in another window or was deleted. Export your current copy before reopening it.".into());
    }
    Ok(Story {
        id: id.into(),
        revision: revision + 1,
        updated_at,
        data,
    })
}

fn remove(db: &Connection, id: &str, revision: i64) -> Result<(), String> {
    validate_id(id)?;
    let count = db
        .execute(
            "DELETE FROM stories WHERE id = ?1 AND revision = ?2",
            params![id, revision],
        )
        .map_err(|e| e.to_string())?;
    if count != 1 {
        return Err(
            "This story changed or was deleted. Refresh the library before deleting.".into(),
        );
    }
    Ok(())
}

fn summaries(db: &Connection) -> Result<Vec<Summary>, String> {
    let mut statement = db
        .prepare("SELECT id FROM stories ORDER BY updated_at DESC, id")
        .map_err(|e| e.to_string())?;
    let ids = statement
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for id in ids {
        let story = read(db, &id.map_err(|e| e.to_string())?)?;
        let replies: Vec<_> = story
            .data
            .messages
            .iter()
            .filter(|m| matches!(m.role, Role::Assistant))
            .collect();
        let words = story.data.scene.split_whitespace().count()
            + replies
                .iter()
                .map(|m| m.content.split_whitespace().count())
                .sum::<usize>();
        let excerpt = if story.data.scene.is_empty() {
            replies
                .first()
                .map(|m| m.content.as_str())
                .unwrap_or(&story.data.notes)
        } else {
            &story.data.scene
        };
        let excerpt = excerpt
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .take(180)
            .collect();
        result.push(Summary {
            id: story.id,
            revision: story.revision,
            title: story.data.title,
            updated_at: story.updated_at,
            words,
            turns: story.data.messages.len() / 2,
            excerpt,
        });
    }
    Ok(result)
}

fn migrate(db: &mut Connection, legacy: Option<Draft>) -> Result<(), String> {
    let tx = db.transaction().map_err(|e| e.to_string())?;
    let imported = tx
        .query_row(
            "SELECT value FROM metadata WHERE key = 'legacy_imported'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if imported.is_none() {
        if let Some(data) = legacy {
            insert(&tx, "legacy-draft", data)?;
        }
        tx.execute(
            "INSERT INTO metadata (key, value) VALUES ('legacy_imported', '1')",
            [],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn initialize_library(
    path: tauri::State<'_, DatabasePath>,
    legacy: Option<Draft>,
) -> Result<Vec<Summary>, String> {
    let mut db = open(&path)?;
    migrate(&mut db, legacy)?;
    summaries(&db)
}
#[tauri::command]
pub fn list_stories(path: tauri::State<'_, DatabasePath>) -> Result<Vec<Summary>, String> {
    summaries(&open(&path)?)
}
#[tauri::command]
pub fn load_story(path: tauri::State<'_, DatabasePath>, id: String) -> Result<Story, String> {
    read(&open(&path)?, &id)
}
#[tauri::command]
pub fn create_story(
    path: tauri::State<'_, DatabasePath>,
    id: String,
    data: Draft,
) -> Result<Story, String> {
    insert(&open(&path)?, &id, data)
}
#[tauri::command]
pub fn save_story(
    path: tauri::State<'_, DatabasePath>,
    id: String,
    revision: i64,
    data: Draft,
) -> Result<Story, String> {
    update(&open(&path)?, &id, revision, data)
}
#[tauri::command]
pub fn delete_story(
    path: tauri::State<'_, DatabasePath>,
    id: String,
    revision: i64,
) -> Result<(), String> {
    remove(&open(&path)?, &id, revision)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn draft(title: &str) -> Draft {
        Draft {
            title: title.into(),
            scene: "A secret garden.".into(),
            notes: "Remember the key.".into(),
            base_url: "http://127.0.0.1:11434".into(),
            model: "gemma3:4b".into(),
            prompt: "Continue".into(),
            messages: Vec::new(),
            context_tokens: 16384,
        }
    }
    #[test]
    fn stories_survive_reopen_and_remain_separate() {
        let file =
            std::env::temp_dir().join(format!("storry-{}-{}.sqlite3", std::process::id(), now()));
        let path = DatabasePath(file.clone());
        {
            let db = open(&path).unwrap();
            insert(&db, "first", draft("Garden")).unwrap();
            insert(&db, "second", draft("Ocean")).unwrap();
            let mut edited = read(&db, "first").unwrap().data;
            edited.scene = "The key was silver.".into();
            update(&db, "first", 1, edited).unwrap();
        }
        let db = open(&path).unwrap();
        assert_eq!(
            read(&db, "first").unwrap().data.scene,
            "The key was silver."
        );
        assert_eq!(read(&db, "second").unwrap().data.title, "Ocean");
        assert_eq!(summaries(&db).unwrap().len(), 2);
        assert!(update(&db, "first", 1, draft("Stale overwrite")).is_err());
        assert!(remove(&db, "first", 1).is_err());
        remove(&db, "first", 2).unwrap();
        assert!(update(&db, "first", 2, draft("Resurrected")).is_err());
        assert_eq!(summaries(&db).unwrap().len(), 1);
        drop(db);
        std::fs::remove_file(file).unwrap();
    }
    #[test]
    fn migration_runs_once_even_after_story_deleted() {
        let mut db = Connection::open_in_memory().unwrap();
        db.execute_batch("CREATE TABLE stories (id TEXT PRIMARY KEY, revision INTEGER, updated_at INTEGER, data TEXT); CREATE TABLE metadata (key TEXT PRIMARY KEY, value TEXT);").unwrap();
        migrate(&mut db, Some(draft("Old draft"))).unwrap();
        migrate(&mut db, Some(draft("Duplicate"))).unwrap();
        assert_eq!(summaries(&db).unwrap().len(), 1);
        remove(&db, "legacy-draft", 1).unwrap();
        migrate(&mut db, Some(draft("Should stay deleted"))).unwrap();
        assert!(summaries(&db).unwrap().is_empty());
    }
}

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbChatMessage {
    #[serde(default)]
    pub id: String,
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub thinking_content: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<serde_json::Value>,
    #[serde(default)]
    pub tool_call_id: Option<String>,
    #[serde(default)]
    pub images: Option<Vec<String>>,
    #[serde(default)]
    pub attachments: Option<serde_json::Value>,
    #[serde(default)]
    pub timestamp: String,
    #[serde(default)]
    pub tokens_count: Option<u64>,
    #[serde(default)]
    pub generation_speed_tps: Option<f64>,
    #[serde(default)]
    pub metrics: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbChatSession {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub model_id: Option<String>,
    #[serde(default)]
    pub model_name: Option<String>,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub messages: Vec<DbChatMessage>,
    #[serde(default)]
    pub is_private: Option<bool>,
    #[serde(default)]
    pub pinned: Option<bool>,
    #[serde(default)]
    pub archived: Option<bool>,
    #[serde(default)]
    pub archived_at: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbPersona {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub tag: String,
    #[serde(default, rename = "iconName")]
    pub icon_name: String,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub description: String,
    pub system_prompt: String,
    #[serde(default = "default_true")]
    pub is_custom: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DbPersonasData {
    pub custom_personas: Vec<DbPersona>,
    pub overrides: HashMap<String, String>,
    pub active_persona_id: Option<String>,
}

pub struct DatabaseService {
    conn: Mutex<Connection>,
    db_path: PathBuf,
}

impl DatabaseService {
    pub fn new() -> Result<Self, String> {
        let db_path = crate::core::config::resolve_path("~/.atena/atena.db");
        Self::init_with_path(db_path)
    }

    pub fn init_with_path(db_path: PathBuf) -> Result<Self, String> {
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                let _ = std::fs::create_dir_all(parent);
            }
        }

        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open SQLite database at {:?}: {}", db_path, e))?;

        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;
            PRAGMA busy_timeout = 5000;

            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                model_id TEXT,
                model_name TEXT,
                project_id TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                is_private INTEGER DEFAULT 0,
                pinned INTEGER DEFAULT 0,
                archived INTEGER DEFAULT 0,
                archived_at TEXT,
                metadata TEXT
            );

            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                thinking_content TEXT,
                tool_calls TEXT,
                tool_call_id TEXT,
                images TEXT,
                attachments TEXT,
                metrics TEXT,
                tokens_count INTEGER,
                generation_speed_tps REAL,
                timestamp TEXT NOT NULL,
                position INTEGER NOT NULL,
                metadata TEXT,
                FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_messages_session_pos ON messages(session_id, position);
            ",
        )
        .map_err(|e| format!("Failed to apply database migrations: {}", e))?;

        // Heal any corrupted updated_at timestamps from previous batch saves by using latest message timestamp
        let _ = conn.execute(
            "UPDATE sessions
             SET updated_at = COALESCE(
                 (SELECT MAX(timestamp) FROM messages WHERE messages.session_id = sessions.id AND timestamp IS NOT NULL AND timestamp != ''),
                 sessions.created_at
             )
             WHERE updated_at IS NOT NULL",
            [],
        );

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS personas (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                tag TEXT,
                icon_name TEXT,
                color TEXT,
                description TEXT,
                system_prompt TEXT NOT NULL,
                is_custom INTEGER DEFAULT 1,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS persona_overrides (
                persona_id TEXT PRIMARY KEY,
                system_prompt TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            ",
        )
        .map_err(|e| format!("Failed to apply database migrations: {}", e))?;

        Ok(Self {
            conn: Mutex::new(conn),
            db_path,
        })
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.db_path
    }

    // ==========================================
    // Sessions & Messages
    // ==========================================

    pub fn get_sessions(&self) -> Result<Vec<DbChatSession>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT id, title, model_id, model_name, project_id, created_at,
                        is_private, pinned, archived, archived_at, metadata, updated_at
                 FROM sessions
                 ORDER BY pinned DESC, updated_at DESC, created_at DESC",
            )
            .map_err(|e| e.to_string())?;

        let session_rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let title: String = row.get(1)?;
                let model_id: Option<String> = row.get(2)?;
                let model_name: Option<String> = row.get(3)?;
                let project_id: Option<String> = row.get(4)?;
                let created_at: String = row.get(5)?;
                let is_private_int: Option<i64> = row.get(6)?;
                let pinned_int: Option<i64> = row.get(7)?;
                let archived_int: Option<i64> = row.get(8)?;
                let archived_at: Option<String> = row.get(9)?;
                let metadata_str: Option<String> = row.get(10)?;
                let updated_at: Option<String> = row.get(11)?;

                Ok((
                    id,
                    title,
                    model_id,
                    model_name,
                    project_id,
                    created_at,
                    is_private_int.map(|v| v != 0),
                    pinned_int.map(|v| v != 0),
                    archived_int.map(|v| v != 0),
                    archived_at,
                    metadata_str,
                    updated_at,
                ))
            })
            .map_err(|e| e.to_string())?;

        let mut msg_stmt = conn
            .prepare(
                "SELECT id, role, content, thinking_content, tool_calls, tool_call_id,
                        images, attachments, metrics, tokens_count, generation_speed_tps,
                        timestamp, metadata
                 FROM messages
                 WHERE session_id = ?1
                 ORDER BY position ASC",
            )
            .map_err(|e| e.to_string())?;

        let mut sessions = Vec::new();

        for s_res in session_rows {
            let (
                id,
                title,
                model_id,
                model_name,
                project_id,
                created_at,
                is_private,
                pinned,
                archived,
                archived_at,
                metadata_str,
                updated_at,
            ) = s_res.map_err(|e| e.to_string())?;

            let mut extra = HashMap::new();
            if let Some(raw) = metadata_str {
                if let Ok(map) = serde_json::from_str::<HashMap<String, serde_json::Value>>(&raw) {
                    extra = map;
                }
            }

            let msg_rows = msg_stmt
                .query_map(params![&id], |row| {
                    let m_id: String = row.get(0)?;
                    let role: String = row.get(1)?;
                    let content: String = row.get(2)?;
                    let thinking_content: Option<String> = row.get(3)?;
                    let tool_calls_str: Option<String> = row.get(4)?;
                    let tool_call_id: Option<String> = row.get(5)?;
                    let images_str: Option<String> = row.get(6)?;
                    let attachments_str: Option<String> = row.get(7)?;
                    let metrics_str: Option<String> = row.get(8)?;
                    let tokens_count: Option<u64> = row.get(9)?;
                    let generation_speed_tps: Option<f64> = row.get(10)?;
                    let timestamp: String = row.get(11)?;
                    let m_metadata_str: Option<String> = row.get(12)?;

                    Ok((
                        m_id,
                        role,
                        content,
                        thinking_content,
                        tool_calls_str,
                        tool_call_id,
                        images_str,
                        attachments_str,
                        metrics_str,
                        tokens_count,
                        generation_speed_tps,
                        timestamp,
                        m_metadata_str,
                    ))
                })
                .map_err(|e| e.to_string())?;

            let mut messages = Vec::new();
            for m_res in msg_rows {
                let (
                    m_id,
                    role,
                    content,
                    thinking_content,
                    tool_calls_str,
                    tool_call_id,
                    images_str,
                    attachments_str,
                    metrics_str,
                    tokens_count,
                    generation_speed_tps,
                    timestamp,
                    m_metadata_str,
                ) = m_res.map_err(|e| e.to_string())?;

                let tool_calls = tool_calls_str.and_then(|s| serde_json::from_str(&s).ok());
                let images = images_str.and_then(|s| serde_json::from_str(&s).ok());
                let attachments = attachments_str.and_then(|s| serde_json::from_str(&s).ok());
                let metrics = metrics_str.and_then(|s| serde_json::from_str(&s).ok());

                let mut m_extra = HashMap::new();
                if let Some(raw) = m_metadata_str {
                    if let Ok(map) = serde_json::from_str::<HashMap<String, serde_json::Value>>(&raw) {
                        m_extra = map;
                    }
                }

                messages.push(DbChatMessage {
                    id: m_id,
                    role,
                    content,
                    thinking_content,
                    tool_calls,
                    tool_call_id,
                    images,
                    attachments,
                    timestamp,
                    tokens_count,
                    generation_speed_tps,
                    metrics,
                    extra: m_extra,
                });
            }

            sessions.push(DbChatSession {
                id,
                title,
                model_id,
                model_name,
                project_id,
                created_at,
                updated_at,
                messages,
                is_private,
                pinned,
                archived,
                archived_at,
                extra,
            });
        }

        Ok(sessions)
    }

    pub fn get_session(&self, session_id: &str) -> Result<Option<DbChatSession>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare(
                "SELECT id, title, model_id, model_name, project_id, created_at,
                        is_private, pinned, archived, archived_at, metadata, updated_at
                 FROM sessions
                 WHERE id = ?1",
            )
            .map_err(|e| e.to_string())?;

        let session_data = stmt
            .query_row(params![session_id], |row| {
                let id: String = row.get(0)?;
                let title: String = row.get(1)?;
                let model_id: Option<String> = row.get(2)?;
                let model_name: Option<String> = row.get(3)?;
                let project_id: Option<String> = row.get(4)?;
                let created_at: String = row.get(5)?;
                let is_private_int: Option<i64> = row.get(6)?;
                let pinned_int: Option<i64> = row.get(7)?;
                let archived_int: Option<i64> = row.get(8)?;
                let archived_at: Option<String> = row.get(9)?;
                let metadata_str: Option<String> = row.get(10)?;
                let updated_at: Option<String> = row.get(11)?;

                Ok((
                    id,
                    title,
                    model_id,
                    model_name,
                    project_id,
                    created_at,
                    is_private_int.map(|v| v != 0),
                    pinned_int.map(|v| v != 0),
                    archived_int.map(|v| v != 0),
                    archived_at,
                    metadata_str,
                    updated_at,
                ))
            })
            .optional()
            .map_err(|e| e.to_string())?;

        let (
            id,
            title,
            model_id,
            model_name,
            project_id,
            created_at,
            is_private,
            pinned,
            archived,
            archived_at,
            metadata_str,
            updated_at,
        ) = match session_data {
            Some(data) => data,
            None => return Ok(None),
        };

        let mut extra = HashMap::new();
        if let Some(raw) = metadata_str {
            if let Ok(map) = serde_json::from_str::<HashMap<String, serde_json::Value>>(&raw) {
                extra = map;
            }
        }

        let mut msg_stmt = conn
            .prepare(
                "SELECT id, role, content, thinking_content, tool_calls, tool_call_id,
                        images, attachments, metrics, tokens_count, generation_speed_tps,
                        timestamp, metadata
                 FROM messages
                 WHERE session_id = ?1
                 ORDER BY position ASC",
            )
            .map_err(|e| e.to_string())?;

        let msg_rows = msg_stmt
            .query_map(params![&id], |row| {
                let m_id: String = row.get(0)?;
                let role: String = row.get(1)?;
                let content: String = row.get(2)?;
                let thinking_content: Option<String> = row.get(3)?;
                let tool_calls_str: Option<String> = row.get(4)?;
                let tool_call_id: Option<String> = row.get(5)?;
                let images_str: Option<String> = row.get(6)?;
                let attachments_str: Option<String> = row.get(7)?;
                let metrics_str: Option<String> = row.get(8)?;
                let tokens_count: Option<u64> = row.get(9)?;
                let generation_speed_tps: Option<f64> = row.get(10)?;
                let timestamp: String = row.get(11)?;
                let m_metadata_str: Option<String> = row.get(12)?;

                Ok((
                    m_id,
                    role,
                    content,
                    thinking_content,
                    tool_calls_str,
                    tool_call_id,
                    images_str,
                    attachments_str,
                    metrics_str,
                    tokens_count,
                    generation_speed_tps,
                    timestamp,
                    m_metadata_str,
                ))
            })
            .map_err(|e| e.to_string())?;

        let mut messages = Vec::new();
        for m_res in msg_rows {
            let (
                m_id,
                role,
                content,
                thinking_content,
                tool_calls_str,
                tool_call_id,
                images_str,
                attachments_str,
                metrics_str,
                tokens_count,
                generation_speed_tps,
                timestamp,
                m_metadata_str,
            ) = m_res.map_err(|e| e.to_string())?;

            let tool_calls = tool_calls_str.and_then(|s| serde_json::from_str(&s).ok());
            let images = images_str.and_then(|s| serde_json::from_str(&s).ok());
            let attachments = attachments_str.and_then(|s| serde_json::from_str(&s).ok());
            let metrics = metrics_str.and_then(|s| serde_json::from_str(&s).ok());

            let mut m_extra = HashMap::new();
            if let Some(raw) = m_metadata_str {
                if let Ok(map) = serde_json::from_str::<HashMap<String, serde_json::Value>>(&raw) {
                    m_extra = map;
                }
            }

            messages.push(DbChatMessage {
                id: m_id,
                role,
                content,
                thinking_content,
                tool_calls,
                tool_call_id,
                images,
                attachments,
                timestamp,
                tokens_count,
                generation_speed_tps,
                metrics,
                extra: m_extra,
            });
        }

        Ok(Some(DbChatSession {
            id,
            title,
            model_id,
            model_name,
            project_id,
            created_at,
            updated_at,
            messages,
            is_private,
            pinned,
            archived,
            archived_at,
            extra,
        }))
    }

    pub fn save_session(&self, session: &DbChatSession) -> Result<(), String> {
        let mut conn = self.conn.lock().map_err(|e| e.to_string())?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;

        Self::save_session_inner(&tx, session)?;

        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn save_sessions_batch(&self, sessions: &[DbChatSession]) -> Result<(), String> {
        let mut conn = self.conn.lock().map_err(|e| e.to_string())?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;

        for session in sessions {
            Self::save_session_inner(&tx, session)?;
        }

        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn save_session_inner(tx: &rusqlite::Transaction, session: &DbChatSession) -> Result<(), String> {
        let now = Utc::now().to_rfc3339();
        let created_at = if session.created_at.is_empty() {
            now.clone()
        } else {
            session.created_at.clone()
        };

        let updated_at = session
            .updated_at
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string())
            .or_else(|| {
                session
                    .messages
                    .iter()
                    .rev()
                    .find(|m| !m.timestamp.trim().is_empty())
                    .map(|m| m.timestamp.clone())
            })
            .unwrap_or_else(|| {
                if !created_at.is_empty() {
                    created_at.clone()
                } else {
                    now.clone()
                }
            });

        let metadata_str = if session.extra.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&session.extra).unwrap_or_default())
        };

        tx.execute(
            "INSERT INTO sessions (
                id, title, model_id, model_name, project_id, created_at, updated_at,
                is_private, pinned, archived, archived_at, metadata
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                model_id = excluded.model_id,
                model_name = excluded.model_name,
                project_id = excluded.project_id,
                updated_at = excluded.updated_at,
                is_private = excluded.is_private,
                pinned = excluded.pinned,
                archived = excluded.archived,
                archived_at = excluded.archived_at,
                metadata = excluded.metadata",
            params![
                &session.id,
                &session.title,
                &session.model_id,
                &session.model_name,
                &session.project_id,
                &created_at,
                &updated_at,
                session.is_private.map(|v| if v { 1 } else { 0 }).unwrap_or(0),
                session.pinned.map(|v| if v { 1 } else { 0 }).unwrap_or(0),
                session.archived.map(|v| if v { 1 } else { 0 }).unwrap_or(0),
                &session.archived_at,
                &metadata_str,
            ],
        )
        .map_err(|e| format!("Error saving session {}: {}", session.id, e))?;

        // Replace messages for this session
        tx.execute("DELETE FROM messages WHERE session_id = ?1", params![&session.id])
            .map_err(|e| format!("Error clearing old messages for session {}: {}", session.id, e))?;

        for (pos, msg) in session.messages.iter().enumerate() {
            let msg_id = if msg.id.is_empty() {
                format!("{}-{}", session.id, pos)
            } else {
                msg.id.clone()
            };

            let tool_calls_str = msg.tool_calls.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
            let images_str = msg.images.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
            let attachments_str = msg.attachments.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
            let metrics_str = msg.metrics.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default());
            let m_metadata_str = if msg.extra.is_empty() {
                None
            } else {
                Some(serde_json::to_string(&msg.extra).unwrap_or_default())
            };
            let timestamp = if msg.timestamp.is_empty() {
                now.clone()
            } else {
                msg.timestamp.clone()
            };

            tx.execute(
                "INSERT INTO messages (
                    id, session_id, role, content, thinking_content, tool_calls, tool_call_id,
                    images, attachments, metrics, tokens_count, generation_speed_tps,
                    timestamp, position, metadata
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                params![
                    &msg_id,
                    &session.id,
                    &msg.role,
                    &msg.content,
                    &msg.thinking_content,
                    &tool_calls_str,
                    &msg.tool_call_id,
                    &images_str,
                    &attachments_str,
                    &metrics_str,
                    msg.tokens_count,
                    msg.generation_speed_tps,
                    &timestamp,
                    pos as i64,
                    &m_metadata_str,
                ],
            )
            .map_err(|e| format!("Error saving message {} in session {}: {}", msg_id, session.id, e))?;
        }

        Ok(())
    }

    pub fn delete_session(&self, session_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM sessions WHERE id = ?1", params![session_id])
            .map_err(|e| format!("Error deleting session {}: {}", session_id, e))?;
        Ok(())
    }

    pub fn delete_sessions_batch(&self, session_ids: &[String]) -> Result<(), String> {
        let mut conn = self.conn.lock().map_err(|e| e.to_string())?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;

        for id in session_ids {
            tx.execute("DELETE FROM sessions WHERE id = ?1", params![id])
                .map_err(|e| format!("Error deleting session {}: {}", id, e))?;
        }

        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    // ==========================================
    // Personas & Overrides
    // ==========================================

    pub fn get_personas_data(&self) -> Result<DbPersonasData, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn
            .prepare("SELECT id, name, tag, icon_name, color, description, system_prompt, is_custom FROM personas")
            .map_err(|e| e.to_string())?;

        let persona_rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let tag: Option<String> = row.get(2)?;
                let icon_name: Option<String> = row.get(3)?;
                let color: Option<String> = row.get(4)?;
                let description: Option<String> = row.get(5)?;
                let system_prompt: String = row.get(6)?;
                let is_custom_int: Option<i64> = row.get(7)?;

                Ok(DbPersona {
                    id,
                    name,
                    tag: tag.unwrap_or_default(),
                    icon_name: icon_name.unwrap_or_default(),
                    color: color.unwrap_or_default(),
                    description: description.unwrap_or_default(),
                    system_prompt,
                    is_custom: is_custom_int.map(|v| v != 0).unwrap_or(true),
                })
            })
            .map_err(|e| e.to_string())?;

        let mut custom_personas = Vec::new();
        for p in persona_rows {
            custom_personas.push(p.map_err(|e| e.to_string())?);
        }

        let mut override_stmt = conn
            .prepare("SELECT persona_id, system_prompt FROM persona_overrides")
            .map_err(|e| e.to_string())?;

        let override_rows = override_stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let prompt: String = row.get(1)?;
                Ok((id, prompt))
            })
            .map_err(|e| e.to_string())?;

        let mut overrides = HashMap::new();
        for o in override_rows {
            let (id, prompt) = o.map_err(|e| e.to_string())?;
            overrides.insert(id, prompt);
        }

        let active_persona_id: Option<String> = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = 'active_persona_id'",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;

        Ok(DbPersonasData {
            custom_personas,
            overrides,
            active_persona_id,
        })
    }

    pub fn save_persona(&self, persona: &DbPersona) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO personas (id, name, tag, icon_name, color, description, system_prompt, is_custom, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                tag = excluded.tag,
                icon_name = excluded.icon_name,
                color = excluded.color,
                description = excluded.description,
                system_prompt = excluded.system_prompt,
                is_custom = excluded.is_custom,
                updated_at = excluded.updated_at",
            params![
                &persona.id,
                &persona.name,
                &persona.tag,
                &persona.icon_name,
                &persona.color,
                &persona.description,
                &persona.system_prompt,
                if persona.is_custom { 1 } else { 0 },
                &now,
            ],
        )
        .map_err(|e| format!("Failed to save persona {}: {}", persona.id, e))?;

        Ok(())
    }

    pub fn delete_persona(&self, persona_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM personas WHERE id = ?1", params![persona_id])
            .map_err(|e| format!("Failed to delete persona {}: {}", persona_id, e))?;
        conn.execute("DELETE FROM persona_overrides WHERE persona_id = ?1", params![persona_id])
            .map_err(|e| format!("Failed to delete persona override for {}: {}", persona_id, e))?;
        Ok(())
    }

    pub fn save_persona_override(&self, persona_id: &str, system_prompt: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO persona_overrides (persona_id, system_prompt, updated_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(persona_id) DO UPDATE SET
                system_prompt = excluded.system_prompt,
                updated_at = excluded.updated_at",
            params![persona_id, system_prompt, &now],
        )
        .map_err(|e| format!("Failed to save persona override {}: {}", persona_id, e))?;

        Ok(())
    }

    pub fn reset_persona_override(&self, persona_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM persona_overrides WHERE persona_id = ?1", params![persona_id])
            .map_err(|e| format!("Failed to reset persona override {}: {}", persona_id, e))?;
        Ok(())
    }

    // ==========================================
    // App Settings Key-Value Store
    // ==========================================

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let res = conn
            .query_row(
                "SELECT value FROM app_settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        Ok(res)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO app_settings (key, value, updated_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = excluded.updated_at",
            params![key, value, &now],
        )
        .map_err(|e| format!("Failed to set setting {}: {}", key, e))?;

        Ok(())
    }

    pub fn get_all_settings(&self) -> Result<HashMap<String, String>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT key, value FROM app_settings")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                let k: String = row.get(0)?;
                let v: String = row.get(1)?;
                Ok((k, v))
            })
            .map_err(|e| e.to_string())?;

        let mut map = HashMap::new();
        for r in rows {
            let (k, v) = r.map_err(|e| e.to_string())?;
            map.insert(k, v);
        }

        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_lifecycle_and_sessions() {
        let temp_dir = std::env::temp_dir().join(format!("atena_test_db_{}", rand::random::<u64>()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let db_file = temp_dir.join("test.db");

        let db = DatabaseService::init_with_path(db_file.clone()).expect("Failed to init DB");

        // 1. Initial sessions should be empty
        let sessions = db.get_sessions().expect("Failed to get sessions");
        assert!(sessions.is_empty());

        // 2. Insert a session with messages
        let session = DbChatSession {
            id: "sess-1".to_string(),
            title: "Test Conversation".to_string(),
            model_id: Some("mlx-community/test-model".to_string()),
            model_name: Some("Test Model".to_string()),
            project_id: None,
            created_at: Utc::now().to_rfc3339(),
            updated_at: None,
            messages: vec![
                DbChatMessage {
                    id: "msg-1".to_string(),
                    role: "user".to_string(),
                    content: "Hello Atena!".to_string(),
                    thinking_content: None,
                    tool_calls: None,
                    tool_call_id: None,
                    images: None,
                    attachments: None,
                    timestamp: Utc::now().to_rfc3339(),
                    tokens_count: Some(5),
                    generation_speed_tps: None,
                    metrics: None,
                    extra: HashMap::new(),
                },
                DbChatMessage {
                    id: "msg-2".to_string(),
                    role: "assistant".to_string(),
                    content: "Hello! How can I assist you?".to_string(),
                    thinking_content: Some("User greeted me warmly.".to_string()),
                    tool_calls: None,
                    tool_call_id: None,
                    images: None,
                    attachments: None,
                    timestamp: Utc::now().to_rfc3339(),
                    tokens_count: Some(12),
                    generation_speed_tps: Some(42.5),
                    metrics: None,
                    extra: HashMap::new(),
                },
            ],
            is_private: Some(false),
            pinned: Some(true),
            archived: Some(false),
            archived_at: None,
            extra: HashMap::new(),
        };

        db.save_session(&session).expect("Failed to save session");

        // 3. Fetch session
        let loaded = db.get_session("sess-1").expect("Failed to get session");
        assert!(loaded.is_some());
        let s = loaded.unwrap();
        assert_eq!(s.title, "Test Conversation");
        assert_eq!(s.messages.len(), 2);
        assert_eq!(s.messages[0].content, "Hello Atena!");
        assert_eq!(s.messages[1].thinking_content, Some("User greeted me warmly.".to_string()));
        assert_eq!(s.pinned, Some(true));

        // 4. Test Personas
        let persona = DbPersona {
            id: "custom-code-guru".to_string(),
            name: "Code Guru".to_string(),
            tag: "DEV".to_string(),
            icon_name: "CodeBracketIcon".to_string(),
            color: "#6366f1".to_string(),
            description: "Expert software architect".to_string(),
            system_prompt: "You are an expert coder.".to_string(),
            is_custom: true,
        };
        db.save_persona(&persona).expect("Failed to save persona");
        db.save_persona_override("general", "Be concise.").expect("Failed to save override");
        db.set_setting("active_persona_id", "custom-code-guru").expect("Failed to set active persona");

        let p_data = db.get_personas_data().expect("Failed to get personas data");
        assert_eq!(p_data.custom_personas.len(), 1);
        assert_eq!(p_data.custom_personas[0].id, "custom-code-guru");
        assert_eq!(p_data.overrides.get("general"), Some(&"Be concise.".to_string()));
        assert_eq!(p_data.active_persona_id, Some("custom-code-guru".to_string()));

        // 5. Test Settings
        db.set_setting("theme", "dark").expect("Failed to set setting");
        let val = db.get_setting("theme").expect("Failed to get setting");
        assert_eq!(val, Some("dark".to_string()));

        // 6. Delete session
        db.delete_session("sess-1").expect("Failed to delete session");
        let after_delete = db.get_session("sess-1").expect("Failed to check deleted session");
        assert!(after_delete.is_none());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}

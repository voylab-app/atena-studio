use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Ephemeral in-memory scratchpad manager.
/// Stores temporary working memory and task notes per session to keep
/// the permanent associative memory graph clean and dense.
#[derive(Clone, Default)]
pub struct SessionScratchpadManager {
    /// Mapping of session_id -> (key -> content)
    store: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
}

impl SessionScratchpadManager {
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Writes or overwrites a key in the session scratchpad
    pub async fn write(&self, session_id: &str, key: &str, content: &str) {
        let mut lock = self.store.write().await;
        let session_map = lock.entry(session_id.to_string()).or_default();
        session_map.insert(key.to_string(), content.to_string());
    }

    /// Appends content to an existing key, or creates it if not present
    pub async fn append(&self, session_id: &str, key: &str, content: &str) {
        let mut lock = self.store.write().await;
        let session_map = lock.entry(session_id.to_string()).or_default();
        let entry = session_map.entry(key.to_string()).or_default();
        if !entry.is_empty() {
            entry.push('\n');
        }
        entry.push_str(content);
    }

    /// Reads a specific key from the session scratchpad
    pub async fn read_key(&self, session_id: &str, key: &str) -> Option<String> {
        let lock = self.store.read().await;
        lock.get(session_id).and_then(|m| m.get(key).cloned())
    }

    /// Reads all key-value entries in the session scratchpad
    pub async fn read_all(&self, session_id: &str) -> HashMap<String, String> {
        let lock = self.store.read().await;
        lock.get(session_id).cloned().unwrap_or_default()
    }

    /// Clears the scratchpad for a given session
    pub async fn clear(&self, session_id: &str) {
        let mut lock = self.store.write().await;
        lock.remove(session_id);
    }

    /// Returns a formatted prompt injection block representing the active scratchpad
    pub async fn format_context_block(&self, session_id: &str) -> Option<String> {
        let entries = self.read_all(session_id).await;
        if entries.is_empty() {
            return None;
        }

        let mut block = String::from(
            "[TASK SCRATCHPAD - EPHEMERAL WORKING MEMORY]\n\
            (Temporary notes for current task only. This is erased after the task. Do not treat as permanent memory.)\n"
        );

        for (k, v) in entries {
            block.push_str(&format!("- {}:\n{}\n\n", k, v.trim()));
        }

        Some(block.trim_end().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scratchpad_lifecycle() {
        let manager = SessionScratchpadManager::new();
        let sess = "test-session-1";

        manager.write(sess, "task", "Investigate failing test").await;
        manager.append(sess, "notes", "Step 1 passed").await;
        manager.append(sess, "notes", "Step 2 failed on line 12").await;

        let task_val = manager.read_key(sess, "task").await;
        assert_eq!(task_val.as_deref(), Some("Investigate failing test"));

        let notes_val = manager.read_key(sess, "notes").await;
        assert_eq!(notes_val.as_deref(), Some("Step 1 passed\nStep 2 failed on line 12"));

        let block = manager.format_context_block(sess).await;
        assert!(block.is_some());
        assert!(block.unwrap().contains("Step 2 failed on line 12"));

        manager.clear(sess).await;
        assert!(manager.read_all(sess).await.is_empty());
        assert!(manager.format_context_block(sess).await.is_none());
    }
}

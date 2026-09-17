// =============================================================================
// services/memory_engine.rs — Procedural Associative Memory Engine
// =============================================================================
//
// Implements:
//   1. Compact binary serialization (manual, zero-alloc where possible)
//   2. High-speed compression (Zstd level 3 / LZ4)
//   3. Graph Engine with O(1) HashMap indexing
//   4. Hot Cache LFU (Least Frequently Used) in RAM
//   5. Synaptic Plasticity (reinforce_edge / apply_decay)
//   6. Spreading Activation (depth-based associative traversal)
//   7. Context synthesizer for LLM system prompts
//
// Serialized binary buffer format (prior to compression):
//   [node_count: u32][nodes...][edge_count: u32][edges...]
//   Where each node has variable size and each edge has a fixed 25-byte layout.

use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::memory::*;

// =============================================================================
// Hot Cache LFU — In-RAM Cache by Access Frequency
// =============================================================================

/// Composite key to identify an edge in cache
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct EdgeKey {
    source_id: u32,
    target_id: u32,
}

/// Cache entry with frequency tracking
#[derive(Debug, Clone)]
struct CacheEntry {
    edge: MemoryEdge,
    frequency: u64,
}

/// LFU Hot Cache with fixed capacity.
///
/// Stores the most frequently accessed edges.
/// When full, evicts the least frequently accessed entry.
#[derive(Debug)]
pub struct HotCache {
    capacity: usize,
    entries: HashMap<EdgeKey, CacheEntry>,
    /// Statistics counters
    pub hits: u64,
    pub misses: u64,
}

impl HotCache {
    /// Creates a new HotCache with the specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: HashMap::with_capacity(capacity),
            hits: 0,
            misses: 0,
        }
    }

    /// Retrieves an edge from cache. Returns Some and increments frequency if found.
    pub fn get(&mut self, source_id: u32, target_id: u32) -> Option<&MemoryEdge> {
        let key = EdgeKey { source_id, target_id };
        if let Some(entry) = self.entries.get_mut(&key) {
            entry.frequency += 1;
            self.hits += 1;
            Some(&entry.edge)
        } else {
            self.misses += 1;
            None
        }
    }

    /// Inserts or updates an edge in cache.
    /// If full, evicts the least frequent entry.
    pub fn insert(&mut self, edge: MemoryEdge) {
        let key = EdgeKey {
            source_id: edge.source_id,
            target_id: edge.target_id,
        };

        // If entry already exists, update the edge and increment frequency
        if let Some(entry) = self.entries.get_mut(&key) {
            entry.edge = edge;
            entry.frequency += 1;
            return;
        }

        // If full, evict the least frequent entry
        if self.entries.len() >= self.capacity {
            self.evict_least_frequent();
        }

        self.entries.insert(key, CacheEntry { edge, frequency: 1 });
    }

    /// Removes the least frequently accessed entry
    fn evict_least_frequent(&mut self) {
        if let Some(min_key) = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.frequency)
            .map(|(key, _)| key.clone())
        {
            self.entries.remove(&min_key);
        }
    }

    /// Returns all neighbor edges of a node currently in cache,
    /// sorted by descending weight
    pub fn get_neighbors(&mut self, source_id: u32) -> Vec<MemoryEdge> {
        let mut neighbors = Vec::new();
        for (key, entry) in self.entries.iter_mut() {
            if key.source_id == source_id {
                entry.frequency += 1;
                neighbors.push(entry.edge.clone());
            }
        }
        neighbors.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap_or(Ordering::Equal));
        self.hits += neighbors.len() as u64;
        neighbors
    }

    /// Invalidates cache entries for a specific edge
    pub fn invalidate(&mut self, source_id: u32, target_id: u32) {
        let key = EdgeKey { source_id, target_id };
        self.entries.remove(&key);
    }

    /// Clears the entire cache
    pub fn clear(&mut self) {
        self.entries.clear();
        self.hits = 0;
        self.misses = 0;
    }

    /// Number of entries currently in cache
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// =============================================================================
// Helper node for priority-queue BFS (Spreading Activation)
// =============================================================================

#[derive(Debug, Clone)]
struct ActivationNode {
    node_id: u32,
    accumulated_weight: f32,
    depth: usize,
    path: Vec<AssociationStep>,
}

impl PartialEq for ActivationNode {
    fn eq(&self, other: &Self) -> bool {
        self.accumulated_weight == other.accumulated_weight
    }
}

impl Eq for ActivationNode {}

impl PartialOrd for ActivationNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ActivationNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Max-heap: higher weight has higher priority
        self.accumulated_weight
            .partial_cmp(&other.accumulated_weight)
            .unwrap_or(Ordering::Equal)
    }
}

// =============================================================================
// MemoryGraphEngine — Core Engine
// =============================================================================

/// Binary Graph Engine with procedural associative memory.
///
/// O(1) Indexing Structures:
///   - `nodes`: HashMap<u32, MemoryNode> — ID lookup
///   - `label_index`: HashMap<String, u32> — reverse lookup by label
///   - `adjacency`: HashMap<u32, Vec<MemoryEdge>> — adjacency list
///
/// Components:
///   - LFU HotCache for frequently accessed edges
///   - Compact binary serialization + Zstd/LZ4 compression
///   - Synaptic plasticity (reinforcement and decay)
pub struct MemoryGraphEngine {
    /// Map of nodes indexed by ID — O(1) lookup
    pub nodes: HashMap<u32, MemoryNode>,
    /// Reverse index: label (lowercase) -> node ID — O(1) lookup by name
    pub label_index: HashMap<String, u32>,
    /// Adjacency list: source_id -> Vec<MemoryEdge>
    pub adjacency: HashMap<u32, Vec<MemoryEdge>>,
    /// Auto-increment for node IDs
    pub next_id: u32,
    /// Hot Cache LFU in RAM
    pub hot_cache: HotCache,
}

impl MemoryGraphEngine {
    /// Creates a new empty graph engine with the specified cache capacity
    pub fn new(cache_capacity: usize) -> Self {
        Self {
            nodes: HashMap::new(),
            label_index: HashMap::new(),
            adjacency: HashMap::new(),
            next_id: 1,
            hot_cache: HotCache::new(cache_capacity),
        }
    }

    /// Default cache capacity: 128 entries
    pub fn with_default_cache() -> Self {
        Self::new(128)
    }

    /// Cleans and sanitizes Markdown tags, tables, headers (#), pipes (|), asterisks, duplicate quotes, and formatting artifacts
    pub fn sanitize_markdown_text(input: &str) -> String {
        let mut s = input.to_string();

        // 1. Remove fenced code blocks (``` ... ```)
        while let Some(start) = s.find("```") {
            if let Some(end) = s[start + 3..].find("```") {
                let actual_end = start + 3 + end + 3;
                s.replace_range(start..actual_end, " ");
            } else {
                s.replace_range(start..start + 3, " ");
            }
        }

        // 2. Convert markdown links [text](url) -> text
        while let Some(start) = s.find('[') {
            if let Some(mid) = s[start..].find("](") {
                let mid_idx = start + mid;
                if let Some(end) = s[mid_idx..].find(')') {
                    let end_idx = mid_idx + end;
                    let link_text = s[start + 1..mid_idx].to_string();
                    s.replace_range(start..=end_idx, &link_text);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // 3. Remove JSON escapes and special characters
        s = s.replace("\\\"", "\"").replace("\\'", "'").replace("\\n", " ").replace("\\t", " ");

        // 4. Remove markdown table divider lines (|---|---|)
        let mut cleaned_lines = Vec::new();
        for line in s.lines() {
            let l = line.trim();
            // Ignore pure table divider lines like |---|---| or |:---|
            let is_table_divider = l.starts_with('|') && l.contains("---");
            if !is_table_divider && !l.is_empty() {
                cleaned_lines.push(l);
            }
        }
        s = cleaned_lines.join(" ");

        // 5. Remove all table pipes (|) and heading hashes (#)
        s = s.replace('|', " ");
        s = s.replace('#', " ");

        // 6. Remove bold, italic, strikethrough, and inline code formatting
        s = s.replace("**", " ").replace("___", " ").replace("__", " ");
        s = s.replace("~~", " ");
        s = s.replace('`', "");
        s = s.replace('*', " ");
        s = s.replace('_', " ");

        // 7. Remove redundant and typographical quotes
        s = s.replace('"', " ").replace('“', " ").replace('”', " ");
        s = s.replace('«', " ").replace('»', " ");

        // 8. Remove simple HTML tags like <br>, <p>, </div>
        while let Some(start) = s.find('<') {
            if let Some(end) = s[start..].find('>') {
                let end_idx = start + end;
                let tag_candidate = &s[start + 1..end_idx];
                if tag_candidate.chars().all(|c| c.is_alphanumeric() || c == '/' || c == ' ' || c == '-' || c == '=' || c == '"' || c == '\'') {
                    s.replace_range(start..=end_idx, " ");
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // 9. Normalize multiple whitespace and dangling punctuation
        let words: Vec<&str> = s.split_whitespace().collect();
        let result = words.join(" ");

        let trimmed = result
            .trim_matches(|c: char| c == '*' || c == '_' || c == '"' || c == '\'' || c == '`' || c == ':' || c == '-' || c == ',' || c == ';' || c == '#' || c == '|')
            .trim();

        // 10. Clean up spaces before commas/periods created by substitutions (e.g. " , " -> ", ")
        let mut final_res = trimmed.replace(" ,", ",").replace(" .", ".").replace(" :", ":");
        while final_res.contains("  ") {
            final_res = final_res.replace("  ", " ");
        }

        final_res
    }

    // =========================================================================
    // CRUD Operations for Nodes and Edges
    // =========================================================================

    /// Adds a new node to the graph with neutral valence (0). Returns the generated ID.
    pub fn add_node(&mut self, type_flag: NodeType, label: &str) -> u32 {
        self.add_node_with_valence(type_flag, label, 0)
    }

    /// Adds a new node with biological valence (-1, 0, +1), sanitizing markdown.
    pub fn add_node_with_valence(&mut self, type_flag: NodeType, label: &str, valence: i8) -> u32 {
        self.add_node_with_valence_and_session(type_flag, label, valence, None)
    }

    /// Adds a new node with biological valence and optional session scope.
    pub fn add_node_with_valence_and_session(
        &mut self,
        type_flag: NodeType,
        label: &str,
        valence: i8,
        session_id: Option<String>,
    ) -> u32 {
        let clean = Self::sanitize_markdown_text(label);
        let final_label = if clean.is_empty() { label.trim().to_string() } else { clean };

        let id = self.next_id;
        self.next_id += 1;

        let now = Self::current_timestamp();
        let node = MemoryNode {
            id,
            type_flag,
            label: final_label.clone(),
            valence,
            created_at: now,
            session_id: session_id.filter(|s| !s.trim().is_empty()),
        };

        self.label_index.insert(final_label.to_lowercase(), id);
        self.nodes.insert(id, node);
        id
    }

    /// Adds an edge between two existing nodes.
    /// The initial weight is 1.0 and access_count starts at 0.
    pub fn add_edge(
        &mut self,
        source_id: u32,
        target_id: u32,
        relation_type: RelationType,
    ) -> Result<(), String> {
        self.add_edge_with_timestamp(source_id, target_id, relation_type, Self::current_timestamp())
    }

    /// Adds an edge between two nodes with an explicit creation/occurrence timestamp.
    pub fn add_edge_with_timestamp(
        &mut self,
        source_id: u32,
        target_id: u32,
        relation_type: RelationType,
        timestamp: u64,
    ) -> Result<(), String> {
        // Validate that both nodes exist
        if !self.nodes.contains_key(&source_id) {
            return Err(format!("Node source_id={} not found", source_id));
        }
        if !self.nodes.contains_key(&target_id) {
            return Err(format!("Node target_id={} not found", target_id));
        }

        // Avoid duplicate edges with the same nodes and relation type
        if let Some(edges) = self.adjacency.get_mut(&source_id) {
            if let Some(existing) = edges.iter_mut().find(|e| e.target_id == target_id && e.relation_type == relation_type) {
                existing.last_accessed = timestamp;
                self.hot_cache.insert(existing.clone());
                return Ok(());
            }
        }

        let edge = MemoryEdge {
            source_id,
            target_id,
            relation_type,
            weight: 1.0,
            access_count: 0,
            last_accessed: timestamp,
            created_at: timestamp,
        };

        // Insert into adjacency list and hot cache
        self.adjacency
            .entry(source_id)
            .or_insert_with(Vec::new)
            .push(edge.clone());
        self.hot_cache.insert(edge);

        Ok(())
    }

    /// Finds a node by label (case-insensitive). O(1) via label_index.
    pub fn find_node_by_label(&self, label: &str) -> Option<&MemoryNode> {
        self.label_index
            .get(&label.to_lowercase())
            .and_then(|id| self.nodes.get(id))
    }

    /// Finds a node by ID. O(1) via HashMap.
    pub fn get_node(&self, id: u32) -> Option<&MemoryNode> {
        self.nodes.get(&id)
    }

    /// Returns the total number of nodes in the graph
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the total number of edges in the graph
    pub fn edge_count(&self) -> usize {
        self.adjacency.values().map(|edges| edges.len()).sum()
    }

    // =========================================================================
    // Synaptic Plasticity — Learning and Forgetting
    // =========================================================================

    /// Reinforces the connection between two nodes (learning by repetition).
    ///
    /// Increments `weight += 0.1`, `access_count += 1`, and updates `last_accessed`.
    /// Also updates the corresponding entry in the Hot Cache.
    pub fn reinforce_edge(&mut self, source_id: u32, target_id: u32) -> Result<(), String> {
        let now = Self::current_timestamp();
        let edges = self
            .adjacency
            .get_mut(&source_id)
            .ok_or_else(|| format!("Nenhuma aresta partindo do nó {}", source_id))?;

        let edge = edges
            .iter_mut()
            .find(|e| e.target_id == target_id)
            .ok_or_else(|| format!("Aresta {} -> {} não encontrada", source_id, target_id))?;

        // Reforço sináptico: aumenta peso e contador
        edge.weight += 0.1;
        edge.access_count += 1;
        edge.last_accessed = now;

        // Update Hot Cache with the reinforced edge
        self.hot_cache.insert(edge.clone());

        Ok(())
    }

    /// Applies synaptic decay (natural forgetting) to all edges.
    ///
    /// Multiplies `weight *= (1.0 - decay_factor)` for each edge.
    /// Edges with `weight < threshold` are pruned.
    /// Recommended value: decay_factor = 0.05 (5% decay)
    pub fn apply_decay(&mut self, decay_factor: f32) {
        let threshold = 0.01_f32;

        for edges in self.adjacency.values_mut() {
            edges.retain_mut(|edge| {
                edge.weight *= 1.0 - decay_factor;
                if edge.weight < threshold {
                    // Pruned edge — remove from cache as well
                    self.hot_cache
                        .invalidate(edge.source_id, edge.target_id);
                    false
                } else {
                    true
                }
            });
        }
    }

    // =========================================================================
    // Spreading Activation — Associative Traversal
    // =========================================================================

    /// Navega o grafo a partir de um nó âncora usando Spreading Activation.
    ///
    /// Usa uma priority queue (max-heap) para explorar primeiro os caminhos
    /// com maior peso acumulado. Consulta o Hot Cache antes do grafo completo.
    ///
    /// # Parâmetros
    /// - `start_label`: Label do nó âncora (case-insensitive)
    /// - `max_depth`: Profundidade máxima de busca
    ///
    /// # Retorna
    /// Vec de caminhos associativos ordenados por peso total decrescente
    pub fn traverse_associations(
        &mut self,
        start_label: &str,
        max_depth: usize,
    ) -> Result<Vec<AssociationPath>, String> {
        let clean_start = start_label.trim();
        if clean_start.is_empty() {
            return Err("Termo de busca vazio".into());
        }

        // 1. Tentar correspondência exata
        let start_node = if let Some(node) = self.find_node_by_label(clean_start) {
            node.clone()
        } else {
            // 2. If exact label not found, look for an anchor node contained inside the phrase
            let lower_query = clean_start.to_lowercase();
            let mut best_match: Option<MemoryNode> = None;
            let mut longest_len = 0;

            // Search for node whose label is contained in query or vice-versa
            for (label, id) in &self.label_index {
                let lbl_lower = label.to_lowercase();
                if lower_query.contains(&lbl_lower) || lbl_lower.contains(&lower_query) {
                    if lbl_lower.len() > longest_len {
                        longest_len = lbl_lower.len();
                        if let Some(node) = self.nodes.get(id) {
                            best_match = Some(node.clone());
                        }
                    }
                }
            }

            // If still not found, search by individual query words (ignoring stop-words)
            if best_match.is_none() {
                let stopwords = ["quem", "qual", "onde", "como", "quando", "que", "esta", "está", "tem", "uma", "um", "filha", "filho", "esposa", "marido", "dela", "dele", "meu", "minha", "nosso", "nossa", "dos", "das", "para", "com", "por"];
                let words: Vec<&str> = lower_query
                    .split(|c: char| !c.is_alphanumeric())
                    .filter(|w| w.len() >= 3 && !stopwords.contains(w))
                    .collect();

                for w in words {
                    for (label, id) in &self.label_index {
                        let lbl_lower = label.to_lowercase();
                        if lbl_lower.contains(w) || w.contains(&lbl_lower) {
                            if let Some(node) = self.nodes.get(id) {
                                best_match = Some(node.clone());
                                break;
                            }
                        }
                    }
                    if best_match.is_some() {
                        break;
                    }
                }
            }

            best_match.ok_or_else(|| format!("Matching node for '{}' not found in memory graph", clean_start))?
        };

        let mut results: Vec<AssociationPath> = Vec::new();
        let mut visited: HashMap<u32, bool> = HashMap::new();
        visited.insert(start_node.id, true);

        // Priority queue (max-heap por peso acumulado)
        let mut heap = BinaryHeap::new();
        let start_ts = if start_node.created_at > 0 { Some(start_node.created_at) } else { None };
        heap.push(ActivationNode {
            node_id: start_node.id,
            accumulated_weight: 1.0,
            depth: 0,
            path: vec![AssociationStep {
                node_label: start_node.label.clone(),
                node_type: start_node.type_flag,
                relation: None,
                edge_weight: None,
                edge_timestamp: start_ts,
            }],
        });

        let mut iterations = 0;
        while let Some(current) = heap.pop() {
            iterations += 1;
            if iterations > 500 {
                break;
            }
            if current.depth >= max_depth {
                // Caminho completo — adicionar aos resultados
                if current.path.len() > 1 {
                    results.push(AssociationPath {
                        steps: current.path,
                        total_weight: current.accumulated_weight,
                    });
                }
                continue;
            }

            // Fetch neighbors: Hot Cache first, then graph
            let neighbors = self.get_neighbors_cached(current.node_id);

            if neighbors.is_empty() && current.path.len() > 1 {
                // Folha — adicionar caminho parcial aos resultados
                results.push(AssociationPath {
                    steps: current.path,
                    total_weight: current.accumulated_weight,
                });
                continue;
            }

            let mut expanded = false;
            for edge in &neighbors {
                if visited.contains_key(&edge.target_id) {
                    continue;
                }
                visited.insert(edge.target_id, true);

                if let Some(target_node) = self.nodes.get(&edge.target_id) {
                    let mut new_path = current.path.clone();
                    let edge_ts = if edge.created_at > 0 { edge.created_at } else { edge.last_accessed };
                    new_path.push(AssociationStep {
                        node_label: target_node.label.clone(),
                        node_type: target_node.type_flag,
                        relation: Some(edge.relation_type),
                        edge_weight: Some(edge.weight),
                        edge_timestamp: if edge_ts > 0 { Some(edge_ts) } else { None },
                    });

                    let new_weight = current.accumulated_weight * edge.weight;

                    heap.push(ActivationNode {
                        node_id: edge.target_id,
                        accumulated_weight: new_weight,
                        depth: current.depth + 1,
                        path: new_path,
                    });
                    expanded = true;
                }
            }

            // Se nenhum vizinho novo foi expandido, emitir caminho parcial
            if !expanded && current.path.len() > 1 {
                results.push(AssociationPath {
                    steps: current.path,
                    total_weight: current.accumulated_weight,
                });
            }
        }

        // Ordenar por peso total decrescente e limitar aos melhores caminhos
        results.sort_by(|a, b| {
            b.total_weight
                .partial_cmp(&a.total_weight)
                .unwrap_or(Ordering::Equal)
        });
        results.truncate(32);

        Ok(results)
    }

    /// Fetches neighbors of a node: queries Hot Cache first, then graph.
    /// Edges found in graph are promoted to cache.
    fn get_neighbors_cached(&mut self, node_id: u32) -> Vec<MemoryEdge> {
        // Tentar Hot Cache primeiro
        let cached = self.hot_cache.get_neighbors(node_id);
        if !cached.is_empty() {
            return cached;
        }

        // Cache miss — search in full graph
        if let Some(edges) = self.adjacency.get(&node_id) {
            let mut result = edges.clone();
            result.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap_or(Ordering::Equal));

            // Promover ao Hot Cache
            for edge in &result {
                self.hot_cache.insert(edge.clone());
            }

            result
        } else {
            Vec::new()
        }
    }

    // =========================================================================
    // Sintetizador de Contexto para LLM
    // =========================================================================

    /// Formata um timestamp de memória para exibição compreensível pela LLM e pelo usuário,
    /// incluindo tanto a data/hora absoluta quanto o tempo decorrido relativo.
    /// Exemplo: "01/09/2026 12:30 (hoje)" ou "15/08/2026 14:20 (há 17 dias)"
    pub fn format_memory_timestamp(ts: u64, current_ts: u64) -> String {
        if ts == 0 {
            return "unknown date".to_string();
        }

        let date_str = chrono::DateTime::from_timestamp(ts as i64, 0)
            .map(|dt| dt.with_timezone(&chrono::Local).format("%d/%m/%Y %H:%M").to_string())
            .unwrap_or_else(|| "invalid date".to_string());

        let relative = if current_ts >= ts {
            let diff = current_ts - ts;
            if diff < 60 {
                "just now".to_string()
            } else if diff < 3600 {
                let m = diff / 60;
                format!("{} min ago", m)
            } else if diff < 86400 {
                let h = diff / 3600;
                format!("today, {} h ago", h)
            } else if diff < 172800 {
                "yesterday".to_string()
            } else if diff < 604800 {
                let d = diff / 86400;
                format!("{} days ago", d)
            } else if diff < 2592000 {
                let w = diff / 604800;
                if w == 1 { "1 week ago".to_string() } else { format!("{} weeks ago", w) }
            } else if diff < 31536000 {
                let mo = diff / 2592000;
                if mo == 1 { "1 month ago".to_string() } else { format!("{} months ago", mo) }
            } else {
                let y = diff / 31536000;
                if y == 1 { "1 year ago".to_string() } else { format!("{} years ago", y) }
            }
        } else {
            "in the future".to_string()
        };

        format!("{} ({})", date_str, relative)
    }

    /// Constructs the contextualized string for LLM prompt injection with precise timestamps.
    ///
    /// Format: `[ACTIVE MEMORY: User -> HAS_PROPERTY: Bought a car [saved at: 15/08/2026 14:20 (17 days ago)]]`
    ///
    /// Multiple paths are separated by ` | `.
    pub fn build_llm_context(paths: &[AssociationPath]) -> String {
        if paths.is_empty() {
            return "[ACTIVE MEMORY: no association found]".to_string();
        }

        let now = Self::current_timestamp();
        let mut context_parts: Vec<String> = Vec::new();

        for path in paths {
            let mut parts: Vec<String> = Vec::new();
            for step in &path.steps {
                if let Some(rel) = &step.relation {
                    if let Some(ts) = step.edge_timestamp {
                        let time_info = Self::format_memory_timestamp(ts, now);
                        parts.push(format!("{}:{} [saved at: {}]", rel.label(), step.node_label, time_info));
                    } else {
                        parts.push(format!("{}:{}", rel.label(), step.node_label));
                    }
                } else {
                    // Anchor node (no relation)
                    parts.push(step.node_label.clone());
                }
            }
            context_parts.push(parts.join(" -> "));
        }

        format!("[ACTIVE MEMORY: {}]", context_parts.join(" | "))
    }

    // =========================================================================
    // Serialização Binária
    // =========================================================================

    /// Serializa todo o grafo em um buffer binário compacto.
    ///
    /// Formato: [node_count: u32][nodes...][edge_count: u32][edges...]
    pub fn serialize_to_bytes(&self) -> Vec<u8> {
        // Estimar tamanho: nodes (variável) + edges (25 bytes cada) + contadores (8 bytes)
        let estimated_size =
            8 + self.nodes.len() * 32 + self.edge_count() * MemoryEdge::BYTE_SIZE;
        let mut buf = Vec::with_capacity(estimated_size);

        // --- Nós ---
        let node_count = self.nodes.len() as u32;
        buf.extend_from_slice(&node_count.to_le_bytes());

        // Serialize nodes in ascending ID order for determinism
        let mut node_ids: Vec<u32> = self.nodes.keys().copied().collect();
        node_ids.sort();
        for id in &node_ids {
            if let Some(node) = self.nodes.get(id) {
                buf.extend_from_slice(&node.to_bytes());
            }
        }

        // --- Arestas ---
        let edge_count = self.edge_count() as u32;
        buf.extend_from_slice(&edge_count.to_le_bytes());

        let mut source_ids: Vec<u32> = self.adjacency.keys().copied().collect();
        source_ids.sort();
        for source_id in &source_ids {
            if let Some(edges) = self.adjacency.get(source_id) {
                for edge in edges {
                    buf.extend_from_slice(&edge.to_bytes());
                }
            }
        }

        buf
    }

    /// Deserializa um buffer binário usando a versão atual do sistema.
    pub fn deserialize_from_bytes(data: &[u8], cache_capacity: usize) -> Result<Self, String> {
        Self::deserialize_from_bytes_version(data, cache_capacity, CURRENT_VERSION)
    }

    /// Deserializa um buffer binário e reconstrói o grafo completo com suporte retrocompatível a versões (v1 e v2).
    pub fn deserialize_from_bytes_version(data: &[u8], cache_capacity: usize, version: u16) -> Result<Self, String> {
        let mut offset = 0;

        // --- Nós ---
        if data.len() < offset + 4 {
            return Err("Buffer muito curto para node_count".into());
        }
        let node_count =
            u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        let mut nodes = HashMap::with_capacity(node_count);
        let mut label_index = HashMap::with_capacity(node_count);
        let mut max_id: u32 = 0;

        for _ in 0..node_count {
            let (node, consumed) = MemoryNode::from_bytes_version(&data[offset..], version)?;
            offset += consumed;
            if node.id > max_id {
                max_id = node.id;
            }
            label_index.insert(node.label.to_lowercase(), node.id);
            nodes.insert(node.id, node);
        }

        // --- Arestas ---
        if data.len() < offset + 4 {
            return Err("Buffer muito curto para edge_count".into());
        }
        let edge_count =
            u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        let mut adjacency: HashMap<u32, Vec<MemoryEdge>> = HashMap::new();
        let mut hot_cache = HotCache::new(cache_capacity);
        let edge_byte_size = if version <= 1 {
            MemoryEdge::BYTE_SIZE_V1
        } else {
            MemoryEdge::BYTE_SIZE
        };

        for _ in 0..edge_count {
            let edge = MemoryEdge::from_bytes_version(&data[offset..], version)?;
            offset += edge_byte_size;

            // Promote edges to Hot Cache during load
            hot_cache.insert(edge.clone());

            adjacency
                .entry(edge.source_id)
                .or_insert_with(Vec::new)
                .push(edge);
        }

        Ok(Self {
            nodes,
            label_index,
            adjacency,
            next_id: max_id + 1,
            hot_cache,
        })
    }

    // =========================================================================
    // Compression and Disk Persistence
    // =========================================================================

    /// Saves graph to compressed .atena binary file.
    ///
    /// Flow: serialize -> compress -> write(header + payload)
    pub fn save_to_compressed_binary(
        &self,
        filepath: &Path,
        compression: CompressionType,
    ) -> Result<usize, String> {
        // 1. Serialize to binary buffer
        let raw_bytes = self.serialize_to_bytes();
        let uncompressed_size = raw_bytes.len() as u64;

        // 2. Compress the buffer
        let compressed = match compression {
            CompressionType::Uncompressed => raw_bytes.clone(),
            CompressionType::Zstd => {
                // Zstd level 3: balanced ratio and high speed
                zstd::encode_all(raw_bytes.as_slice(), 3)
                    .map_err(|e| format!("Zstd compression error: {}", e))?
            }
            CompressionType::Lz4 => {
                // LZ4: ultra-fast compression
                lz4_flex::compress_prepend_size(&raw_bytes)
            }
        };

        // 3. Assemble header
        let header = FileHeader {
            magic: MAGIC_BYTES,
            version: CURRENT_VERSION,
            compression_type: compression,
            uncompressed_size,
        };

        // 4. Write: header (15 bytes) + compressed payload
        let header_bytes = header.to_bytes();
        let total_size = FileHeader::SIZE + compressed.len();
        let mut output = Vec::with_capacity(total_size);
        output.extend_from_slice(&header_bytes);
        output.extend_from_slice(&compressed);

        std::fs::write(filepath, &output)
            .map_err(|e| format!("Error writing file '{}': {}", filepath.display(), e))?;

        Ok(total_size)
    }

    /// Loads graph from a compressed .atena binary file.
    ///
    /// Flow: read -> validate header -> decompress -> deserialize
    pub fn load_from_compressed_binary(
        filepath: &Path,
        cache_capacity: usize,
    ) -> Result<Self, String> {
        // 1. Read complete file
        let file_data = std::fs::read(filepath)
            .map_err(|e| format!("Error reading file '{}': {}", filepath.display(), e))?;

        if file_data.len() < FileHeader::SIZE {
            return Err("Arquivo muito pequeno para conter header válido".into());
        }

        // 2. Parsear e validar header
        let header = FileHeader::from_bytes(&file_data[..FileHeader::SIZE])?;

        // 3. Extrair payload comprimido
        let payload = &file_data[FileHeader::SIZE..];

        // 4. Descomprimir
        let raw_bytes = match header.compression_type {
            CompressionType::Uncompressed => payload.to_vec(),
            CompressionType::Zstd => {
                zstd::decode_all(payload)
                    .map_err(|e| format!("Zstd decompression error: {}", e))?
            }
            CompressionType::Lz4 => {
                lz4_flex::decompress_size_prepended(payload)
                    .map_err(|e| format!("LZ4 decompression error: {}", e))?
            }
        };

        // Validar tamanho descomprimido
        if raw_bytes.len() as u64 != header.uncompressed_size {
            return Err(format!(
                "Tamanho descomprimido diverge: esperado {} bytes, obtido {}",
                header.uncompressed_size,
                raw_bytes.len()
            ));
        }

        // 5. Deserializar o grafo com suporte à versão do header
        Self::deserialize_from_bytes_version(&raw_bytes, cache_capacity, header.version)
    }

    /// Finds existing node by label or creates one (sanitizing markdown)
    pub fn get_or_create_node(&mut self, type_flag: NodeType, label: &str) -> u32 {
        self.get_or_create_node_with_valence(type_flag, label, 0)
    }

    /// Finds existing node by label or creates one with specified valence
    pub fn get_or_create_node_with_valence(&mut self, type_flag: NodeType, label: &str, valence: i8) -> u32 {
        self.get_or_create_node_with_valence_and_session(type_flag, label, valence, None)
    }

    /// Finds existing node by label or creates one with specified valence and session scope
    pub fn get_or_create_node_with_valence_and_session(
        &mut self,
        type_flag: NodeType,
        label: &str,
        valence: i8,
        session_id: Option<&str>,
    ) -> u32 {
        let clean = Self::sanitize_markdown_text(label);
        let search_label = if clean.is_empty() { label.trim() } else { &clean };
        if let Some(id) = self.label_index.get(&search_label.to_lowercase()) {
            let existing_id = *id;
            if let Some(node) = self.nodes.get_mut(&existing_id) {
                if valence != 0 {
                    node.valence = valence;
                }
                if let Some(sid) = session_id {
                    let s_trimmed = sid.trim();
                    if !s_trimmed.is_empty() {
                        node.session_id = Some(s_trimmed.to_string());
                    }
                }
            }
            existing_id
        } else {
            self.add_node_with_valence_and_session(
                type_flag,
                search_label,
                valence,
                session_id.map(|s| s.to_string()),
            )
        }
    }

    /// Updates the scope of a node (Global if None/empty, or Private if Some(session_id))
    pub fn update_node_scope(&mut self, node_id: u32, session_id: Option<String>) -> Result<(), String> {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.session_id = session_id.filter(|s| !s.trim().is_empty());
            let _ = self.auto_persist_default();
            Ok(())
        } else {
            Err(format!("Node with ID {} not found", node_id))
        }
    }

    /// Removes a node and all connecting edges
    pub fn delete_node(&mut self, id: u32) -> Result<(), String> {
        if let Some(node) = self.nodes.remove(&id) {
            self.label_index.remove(&node.label.to_lowercase());
            self.adjacency.remove(&id);

            // Remove references as target
            for edges in self.adjacency.values_mut() {
                edges.retain(|e| e.target_id != id);
            }
            self.hot_cache.clear();
            let _ = self.auto_persist_default();
            Ok(())
        } else {
            Err(format!("Node id={} not found", id))
        }
    }

    /// Checks if a node has any active incoming or outgoing connections
    pub fn has_any_connections(&self, node_id: u32) -> bool {
        if let Some(edges) = self.adjacency.get(&node_id) {
            if !edges.is_empty() {
                return true;
            }
        }
        for edges in self.adjacency.values() {
            if edges.iter().any(|e| e.target_id == node_id) {
                return true;
            }
        }
        false
    }

    /// Executes forgetting or deletion of a memory node/edge requested by AI
    pub fn apply_forget_rule(&mut self, subj: &str, prop: &str, learned: &mut Vec<String>) {
        let subj_clean = subj.trim();
        let prop_clean = prop.trim();

        if subj_clean.is_empty() && prop_clean.is_empty() {
            return;
        }

        // Case 1: Subject provided and property is "*" or empty -> Remove subject node completely
        if !subj_clean.is_empty() && (prop_clean.is_empty() || prop_clean == "*") {
            if let Some(node) = self.find_node_by_label(subj_clean) {
                let id = node.id;
                let _ = self.delete_node(id);
                learned.push(format!("🗑️ [FORGOTTEN ENTITY]: {}", subj_clean));
            }
            return;
        }

        // Case 2: Both subject and property provided -> Remove edge from subject to matching property
        if !subj_clean.is_empty() && !prop_clean.is_empty() {
            if let Some(subj_node) = self.find_node_by_label(subj_clean) {
                let subj_id = subj_node.id;
                let prop_lower = prop_clean.to_lowercase();
                let mut removed_target_ids = Vec::new();

                if let Some(edges) = self.adjacency.get_mut(&subj_id) {
                    edges.retain(|e| {
                        if let Some(t_node) = self.nodes.get(&e.target_id) {
                            let t_lower = t_node.label.to_lowercase();
                            if t_lower == prop_lower || t_lower.contains(&prop_lower) || prop_lower.contains(&t_lower) {
                                removed_target_ids.push(e.target_id);
                                return false;
                            }
                        }
                        true
                    });
                }

                for t_id in removed_target_ids {
                    let t_label = self.nodes.get(&t_id).map(|n| n.label.clone()).unwrap_or_default();
                    learned.push(format!("🗑️ [FORGOTTEN MEMORY]: {} -> {}", subj_clean, t_label));
                    // If target node has no remaining connections, prune orphan node
                    if !self.has_any_connections(t_id) {
                        let _ = self.delete_node(t_id);
                    }
                }
                return;
            }
        }

        // Case 3: Only property provided -> Remove nodes whose label matches/contains property
        if subj_clean.is_empty() && !prop_clean.is_empty() {
            let prop_lower = prop_clean.to_lowercase();
            let matching_ids: Vec<u32> = self.nodes
                .iter()
                .filter(|(_, n)| n.label.to_lowercase() == prop_lower || n.label.to_lowercase().contains(&prop_lower))
                .map(|(id, _)| *id)
                .collect();

            for id in matching_ids {
                let lbl = self.nodes.get(&id).map(|n| n.label.clone()).unwrap_or_default();
                let _ = self.delete_node(id);
                learned.push(format!("🗑️ [DELETED RECORD]: {}", lbl));
            }
        }
    }

    /// Clears associative memory, Hippocampus buffer, and episodic chain journals
    pub fn clear_all(&mut self) {
        self.nodes.clear();
        self.label_index.clear();
        self.adjacency.clear();
        self.next_id = 1;
        self.hot_cache.clear();
        let _ = self.auto_persist_default();
        let _ = Self::clear_vigilia_buffer();
        let _ = Self::save_skills(&[]);
        let _ = Self::clear_episodios();
    }

    /// Returns all nodes and edges for neural visualizer rendering
    pub fn get_full_graph(&self) -> FullGraphData {
        let mut nodes_vec: Vec<MemoryNode> = self.nodes.values().cloned().collect();
        nodes_vec.sort_by_key(|n| n.id);

        let mut edges_vec: Vec<MemoryEdge> = Vec::new();
        for edges in self.adjacency.values() {
            for edge in edges {
                edges_vec.push(edge.clone());
            }
        }

        FullGraphData {
            nodes: nodes_vec,
            edges: edges_vec,
            stats: self.stats(),
        }
    }

    /// Exports the complete graph in structured Markdown format
    pub fn export_graph_markdown(&self) -> String {
        let mut md = String::new();
        let now = Self::current_timestamp();
        let date_str = chrono::DateTime::from_timestamp(now as i64, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| now.to_string());

        md.push_str("# Associative Memory Graph — Atena Studio\n\n");
        md.push_str(&format!("- **Export Date:** {}\n", date_str));
        md.push_str(&format!("- **Total Nodes:** {}\n", self.nodes.len()));
        md.push_str(&format!("- **Total Synapses (Connections):** {}\n\n", self.edge_count()));

        md.push_str("## 🧠 Entities and Concepts (Nodes)\n\n");
        md.push_str("| ID | Label / Entity | Type | Valence | Scope |\n");
        md.push_str("| :--- | :--- | :--- | :--- | :--- |\n");

        let mut sorted_nodes: Vec<&MemoryNode> = self.nodes.values().collect();
        sorted_nodes.sort_by_key(|n| n.id);

        for n in &sorted_nodes {
            let valence_str = match n.valence {
                1 => "🟢 Positive (+1)",
                -1 => "🔴 Inhibition (-1)",
                _ => "⚪ Neutral (0)",
            };
            let scope_str = match &n.session_id {
                Some(sid) if !sid.is_empty() => format!("🔒 Private (`{}`)", sid),
                _ => "🌐 Global".to_string(),
            };
            md.push_str(&format!(
                "| `{}` | **{}** | `{}` | {} | {} |\n",
                n.id,
                n.label.replace('|', "\\|"),
                n.type_flag.name(),
                valence_str,
                scope_str
            ));
        }

        md.push_str("\n## ⚡ Neural Connections (Synapses)\n\n");
        md.push_str("| Source | Relation | Target | Weight | Accesses |\n");
        md.push_str("| :--- | :--- | :--- | :--- | :--- |\n");

        let mut sorted_sources: Vec<u32> = self.adjacency.keys().copied().collect();
        sorted_sources.sort();

        for src_id in sorted_sources {
            if let Some(edges) = self.adjacency.get(&src_id) {
                let src_label = self.nodes.get(&src_id).map(|n| n.label.as_str()).unwrap_or("Unknown");
                for edge in edges {
                    let tgt_label = self.nodes.get(&edge.target_id).map(|n| n.label.as_str()).unwrap_or("Unknown");
                    md.push_str(&format!(
                        "| **{}** | `{}` | **{}** | `{:.2}` | `{}` |\n",
                        src_label.replace('|', "\\|"),
                        edge.relation_type.name(),
                        tgt_label.replace('|', "\\|"),
                        edge.weight,
                        edge.access_count
                    ));
                }
            }
        }

        md
    }

    /// Exporta o grafo completo em formato JSON identado
    pub fn export_graph_json(&self) -> Result<String, String> {
        let full = self.get_full_graph();
        serde_json::to_string_pretty(&full).map_err(|e| format!("Error serializing JSON: {}", e))
    }

    // =========================================================================
    // Modular Lobe Persistence (~/.atena/brain/)
    // =========================================================================

    /// Root directory for modular brain topology
    pub fn brain_root_path() -> std::path::PathBuf {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        std::path::Path::new(&home).join(".atena").join("brain")
    }

    /// Ensures the complete brain lobe directory topology exists
    pub fn ensure_brain_directories() -> Result<std::path::PathBuf, String> {
        let root = Self::brain_root_path();
        let lobes = [
            BrainLobe::Hipocampo,
            BrainLobe::Temporal,
            BrainLobe::Parietal,
            BrainLobe::Prefrontal,
            BrainLobe::Occipital,
        ];

        for lobe in lobes {
            let lobe_dir = root.join(lobe.dir_name());
            std::fs::create_dir_all(&lobe_dir)
                .map_err(|e| format!("Error creating brain lobe directory '{}': {}", lobe_dir.display(), e))?;
        }

        let episodios_dir = root.join("episodios");
        std::fs::create_dir_all(&episodios_dir)
            .map_err(|e| format!("Error creating episodes directory '{}': {}", episodios_dir.display(), e))?;

        Ok(root)
    }

    /// Caminho padrão para o arquivo legado .atena
    pub fn default_memory_file_path() -> std::path::PathBuf {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        let dir = std::path::Path::new(&home).join(".atena");
        let _ = std::fs::create_dir_all(&dir);
        dir.join("memory.atena")
    }

    /// Classifies a node into its corresponding functional brain lobe
    pub fn classify_node_lobe(node: &MemoryNode) -> BrainLobe {
        match node.type_flag {
            NodeType::Container => BrainLobe::Parietal,
            NodeType::Action => BrainLobe::Prefrontal,
            NodeType::RuleOrAlert => BrainLobe::Prefrontal,
            NodeType::Attribute => {
                let lbl = node.label.to_lowercase();
                if lbl.contains("cor ") || lbl.contains("color") || lbl.contains("olhos ") || lbl.contains("eyes") || lbl.contains("cabelos ") || lbl.contains("hair") || lbl.contains("pele ") || lbl.contains("skin") || lbl.contains("visual") {
                    BrainLobe::Occipital
                } else if lbl.contains("prefere ") || lbl.contains("prefers") || lbl.contains("prefer") || lbl.contains("regra") || lbl.contains("rule") || lbl.contains("estilo") || lbl.contains("style") || lbl.contains("evitar") || lbl.contains("avoid") {
                    BrainLobe::Prefrontal
                } else {
                    BrainLobe::Temporal
                }
            }
            NodeType::Object => {
                let lbl = node.label.to_lowercase();
                if lbl.contains("gaveta") || lbl.contains("drawer") || lbl.contains("caixa") || lbl.contains("box") || lbl.contains("lápis") || lbl.contains("pencil") || lbl.contains("chave") || lbl.contains("key") || lbl.contains("passaporte") || lbl.contains("passport") || lbl.contains("objeto") || lbl.contains("object") || lbl.contains("item") {
                    BrainLobe::Parietal
                } else {
                    BrainLobe::Temporal
                }
            }
        }
    }

    // =========================================================================
    // Hippocampus: Wakefulness Buffer and Sleep Consolidation / Pruning
    // =========================================================================

    /// Records an event in the wakefulness buffer (Hippocampus lobe)
    pub fn record_vigilia_event(
        speaker: &str,
        text: &str,
        forced_valence: Option<i8>,
        forced_category: Option<&str>,
    ) -> Result<VigiliaEvent, String> {
        let root = Self::ensure_brain_directories()?;
        let hipo_dir = root.join("hipocampo");
        std::fs::create_dir_all(&hipo_dir).map_err(|e| e.to_string())?;
        let diario_path = hipo_dir.join("diario_vigilia.atena");

        let text_trimmed = text.trim();
        if text_trimmed.is_empty() {
            return Err("Texto vazio para evento de vigília".into());
        }

        // Biological Valence Heuristic Detection
        let lower = text_trimmed.to_lowercase();
        let (detected_valence, detected_category) = if let Some(v) = forced_valence {
            (v, forced_category.unwrap_or("custom").to_string())
        } else if lower.contains("deu certo")
            || lower.contains("excelente")
            || lower.contains("perfeito")
            || lower.contains("muito bom")
            || lower.contains("parabéns")
            || lower.contains("adorou")
            || lower.contains("funcionou")
            || lower.contains("gostei")
        {
            (1i8, "recompensa".to_string())
        } else if lower.contains("deu erro")
            || lower.contains("não faça")
            || lower.contains("nunca faça")
            || lower.contains("não use")
            || lower.contains("falhou")
            || lower.contains("errado")
            || lower.contains("bug")
            || lower.contains("incorreto")
            || lower.contains("evite")
        {
            (-1i8, "erro_alerta".to_string())
        } else if lower.contains("prefiro") || lower.contains("sempre use") || lower.contains("regra") {
            (1i8, "preferencia".to_string())
        } else {
            (0i8, "dialogo".to_string())
        };

        let clean_text = Self::sanitize_markdown_text(text_trimmed);
        let final_text = if clean_text.is_empty() { text_trimmed.to_string() } else { clean_text };
        let bounded_text = if final_text.len() > 800 {
            format!("{}...", &final_text[..800])
        } else {
            final_text
        };

        let now = Self::current_timestamp();
        let event = VigiliaEvent {
            id: now * 1000 + (now % 1000),
            timestamp: now,
            speaker: speaker.to_string(),
            text: bounded_text,
            valence: detected_valence,
            salience: if detected_valence != 0 { 0.9 } else { 0.5 },
            category: detected_category,
        };

        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&diario_path)
            .map_err(|e| format!("Error opening diario_vigilia.atena: {}", e))?;

        if let Ok(json_line) = serde_json::to_string(&event) {
            let _ = writeln!(file, "{}", json_line);
        }

        Ok(event)
    }

    /// Loads all recorded events from the Hippocampus wakefulness buffer
    pub fn load_vigilia_buffer() -> Vec<VigiliaEvent> {
        let root = Self::brain_root_path();
        let diario_path = root.join("hipocampo").join("diario_vigilia.atena");
        if !diario_path.exists() {
            return Vec::new();
        }

        let mut events = Vec::new();
        if let Ok(content) = std::fs::read_to_string(&diario_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    if let Ok(evt) = serde_json::from_str::<VigiliaEvent>(trimmed) {
                        events.push(evt);
                    }
                }
            }
        }
        events
    }

    /// Clears the wakefulness buffer after consolidation
    pub fn clear_vigilia_buffer() -> Result<(), String> {
        let root = Self::brain_root_path();
        let diario_path = root.join("hipocampo").join("diario_vigilia.atena");
        if diario_path.exists() {
            let _ = std::fs::write(&diario_path, "");
        }
        Ok(())
    }

    /// Executa o Ciclo de Sono e Poda Sináptica (Sleep Consolidation & Pruning)
    pub fn run_sleep_cycle(&mut self) -> SleepConsolidationReport {
        let events = Self::load_vigilia_buffer();
        let now = Self::current_timestamp();

        let mut report = SleepConsolidationReport {
            timestamp: now,
            events_processed: events.len(),
            noise_discarded: 0,
            facts_consolidated: 0,
            rules_created: 0,
            synapses_reinforced: 0,
            synapses_pruned: 0,
            positive_count: 0,
            negative_count: 0,
            details: Vec::new(),
        };

        if events.is_empty() {
            report.details.push("Nenhum evento no Hipocampo para consolidação. Aplicando decaimento natural...".to_string());
            report.synapses_pruned = self.prune_weak_synapses(0.20);
            let _ = self.auto_persist_default();
            return report;
        }

        for evt in &events {
            let txt = evt.text.trim();
            let lower = txt.to_lowercase();

            // 1. Noise Filtering (Biological pruning of ephemeral greetings)
            let is_noise = evt.valence == 0 && (
                txt.len() < 5
                || lower == "oi"
                || lower == "olá"
                || lower == "ola"
                || lower == "bom dia"
                || lower == "boa tarde"
                || lower == "boa noite"
                || lower == "obrigado"
                || lower == "valeu"
                || lower == "ok"
                || lower == "beleza"
                || lower == "tchau"
                || lower == "sim"
                || lower == "não"
            );

            if is_noise {
                report.noise_discarded += 1;
                continue;
            }

            // 2. Processamento por Valência Biológica
            if evt.valence > 0 {
                report.positive_count += 1;
                let learned = self.learn_from_text(txt);
                report.facts_consolidated += learned.len();

                for item in &learned {
                    report.details.push(format!("💚 [RECOMPENSA] Consolidado: {}", item));
                }
                report.synapses_reinforced += 1;
            } else if evt.valence < 0 {
                report.negative_count += 1;
                let rule_label = if txt.len() > 60 {
                    format!("Evitar: {}", &txt[..57])
                } else {
                    format!("Evitar: {}", txt)
                };

                let rule_id = self.add_node_with_valence(NodeType::RuleOrAlert, &rule_label, -1);
                report.rules_created += 1;
                report.details.push(format!("⚠️ [SALVAGUARDA] Regra inibitória criada: {}", rule_label));

                if let Some(atena_node) = self.find_node_by_label("Atena") {
                    let atena_id = atena_node.id;
                    let _ = self.add_edge(atena_id, rule_id, RelationType::AvoidAction);
                }
            } else {
                let learned = self.learn_from_text(txt);
                if !learned.is_empty() {
                    report.facts_consolidated += learned.len();
                    for item in &learned {
                        report.details.push(format!("🧠 [FATO] Consolidado: {}", item));
                    }
                }
            }
        }

        // 3. Active Synaptic Pruning (Decay and removal of weak edges)
        self.apply_decay(0.05);
        report.synapses_pruned = self.prune_weak_synapses(0.20);
        report.details.push(format!("✂️ [PODA] {} sinapses em desuso podadas com sucesso", report.synapses_pruned));

        // 4. Save modular brain and clear consolidated wakefulness buffer
        let _ = self.auto_persist_default();
        let _ = Self::clear_vigilia_buffer();

        report
    }

    /// Prunes edges with weight below threshold (Synaptic Pruning)
    pub fn prune_weak_synapses(&mut self, min_weight: f32) -> usize {
        let mut pruned_count = 0;
        for (_src, edges) in self.adjacency.iter_mut() {
            let initial_len = edges.len();
            edges.retain(|edge| edge.weight >= min_weight || edge.access_count >= 3);
            pruned_count += initial_len - edges.len();
        }
        pruned_count
    }

    /// Performs deep connection analysis, deduplication, decay, and pruning of orphan nodes
    pub fn optimize_and_prune_graph(&mut self) -> GraphOptimizationReport {
        let now = Self::current_timestamp();
        let nodes_before = self.nodes.len();
        let edges_before = self.edge_count();
        let mut details = Vec::new();

        // 1. Cleanup of corrupted nodes (malformed XML tags or labels > 100 chars)
        let bad_node_ids: Vec<u32> = self
            .nodes
            .iter()
            .filter(|(_, n)| {
                n.label.contains("<memoria")
                    || n.label.contains("<memory")
                    || n.label.contains("<habilidade")
                    || n.label.contains("<skill")
                    || n.label.contains("<forget")
                    || n.label.contains("<esquecer")
                    || n.label.contains("<remover")
                    || n.label.contains("propriedade=")
                    || n.label.contains("sujeito=")
                    || n.label.contains("tipo=")
                    || n.label.contains("valencia=")
                    || n.label.len() > 180
            })
            .map(|(id, _)| *id)
            .collect();

        let corrupted_removed = bad_node_ids.len();
        for id in bad_node_ids {
            let _ = self.delete_node(id);
        }
        if corrupted_removed > 0 {
            details.push(format!("🧹 [LIMPEZA]: {} nós corrompidos ou fragmentados foram purgados", corrupted_removed));
        }

        // 2. Edge / Synapse Deduplication
        let mut duplicates_removed = 0;
        for (_src, edges) in self.adjacency.iter_mut() {
            let mut seen = std::collections::HashSet::new();
            let before_len = edges.len();
            edges.retain(|e| {
                let key = (e.target_id, e.relation_type);
                seen.insert(key)
            });
            duplicates_removed += before_len - edges.len();
        }
        if duplicates_removed > 0 {
            details.push(format!("🔗 [DESDUPLICAÇÃO]: {} conexões duplicadas unificadas", duplicates_removed));
        }

        // 3. Natural Decay & Pruning of Weak Synapses
        self.apply_decay(0.05);
        let synapses_pruned = self.prune_weak_synapses(0.20);
        if synapses_pruned > 0 {
            details.push(format!("✂️ [PODA SINÁPTICA]: {} conexões fracas ou em desuso foram podadas", synapses_pruned));
        }

        // 4. Pruning of Orphan / Isolated Nodes (Attributes without connections)
        let mut connected_node_ids = std::collections::HashSet::new();
        for (src_id, edges) in &self.adjacency {
            if !edges.is_empty() {
                connected_node_ids.insert(*src_id);
                for e in edges {
                    connected_node_ids.insert(e.target_id);
                }
            }
        }

        let orphan_ids: Vec<u32> = self
            .nodes
            .iter()
            .filter(|(id, n)| {
                !connected_node_ids.contains(id) && (n.type_flag == NodeType::Attribute || n.type_flag == NodeType::Action)
            })
            .map(|(id, _)| *id)
            .collect();

        let orphan_count = orphan_ids.len();
        for id in orphan_ids {
            let _ = self.delete_node(id);
        }
        if orphan_count > 0 {
            details.push(format!("🍂 [PODA DE ÓRFÃOS]: {} nós de atributos desconectados foram removidos", orphan_count));
        }

        // 5. Weight Normalization
        for (_src, edges) in self.adjacency.iter_mut() {
            for e in edges.iter_mut() {
                e.weight = e.weight.clamp(0.1, 1.0);
            }
        }

        // 6. Persistência Imediata no Disco Modular
        let _ = self.auto_persist_default();
        let _ = Self::clear_vigilia_buffer();

        let nodes_after = self.nodes.len();
        let edges_after = self.edge_count();

        details.push(format!("✨ [STATUS FINAL]: Grafo estabilizado com {} nós e {} conexões ativas", nodes_after, edges_after));

        GraphOptimizationReport {
            timestamp: now,
            nodes_before,
            nodes_after,
            edges_before,
            edges_after,
            corrupted_removed,
            duplicates_removed,
            synapses_pruned,
            orphan_nodes_pruned: orphan_count,
            details,
        }
    }

    /// Aplica as decisões cognitivas tomadas pela IA no Ciclo de Sono
    pub fn apply_ai_sleep_consolidation(
        &mut self,
        events: &[VigiliaEvent],
        ai_response: &str,
    ) -> SleepConsolidationReport {
        let now = Self::current_timestamp();
        let mut report = SleepConsolidationReport {
            timestamp: now,
            events_processed: events.len(),
            noise_discarded: 0,
            facts_consolidated: 0,
            rules_created: 0,
            synapses_reinforced: 0,
            synapses_pruned: 0,
            positive_count: 0,
            negative_count: 0,
            details: Vec::new(),
        };

        // Tentar extrair o bloco JSON da resposta da IA
        let json_str = if let Some(start) = ai_response.find("```json") {
            let rest = &ai_response[start + 7..];
            if let Some(end) = rest.find("```") {
                rest[..end].trim()
            } else {
                rest.trim()
            }
        } else if let Some(start) = ai_response.find('{') {
            if let Some(end) = ai_response.rfind('}') {
                &ai_response[start..=end]
            } else {
                ai_response.trim()
            }
        } else {
            ai_response.trim()
        };

        if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
            // 1. Processar ruídos descartados pela IA
            if let Some(noises) = val.get("noise_discarded").and_then(|v| v.as_array()) {
                report.noise_discarded += noises.len();
                for n in noises {
                    if let Some(s) = n.as_str() {
                        report.details.push(format!("🗑️ [IA DESCARTOR RUÍDO]: {}", s));
                    }
                }
            }

            // 2. Processar fatos extraídos pela IA
            if let Some(facts) = val.get("facts").and_then(|v| v.as_array()) {
                for f in facts {
                    let raw_subj = f.get("subject").and_then(|v| v.as_str()).unwrap_or("").trim();
                    let raw_prop = f.get("property").and_then(|v| v.as_str()).unwrap_or("").trim();
                    let valence = f.get("valence").and_then(|v| v.as_i64()).unwrap_or(0) as i8;

                    let subj = Self::sanitize_markdown_text(raw_subj);
                    let prop = Self::sanitize_markdown_text(raw_prop);

                    let lower_subj = subj.to_lowercase();
                    let is_noise = lower_subj == "ele"
                        || lower_subj == "ela"
                        || lower_subj == "dele"
                        || lower_subj == "dela"
                        || lower_subj == "qual"
                        || lower_subj == "quem"
                        || lower_subj == "o"
                        || lower_subj == "a"
                        || lower_subj == "se";

                    if !subj.is_empty() && !prop.is_empty() && subj.len() >= 2 && prop.len() >= 2 && !is_noise {
                        let subj_id = self.get_or_create_node(NodeType::Object, &subj);
                        let attr_id = self.get_or_create_node(NodeType::Attribute, &prop);
                        let _ = self.add_edge(subj_id, attr_id, RelationType::HasProperty);
                        report.facts_consolidated += 1;

                        if valence > 0 {
                            report.positive_count += 1;
                            let _ = self.reinforce_edge(subj_id, attr_id);
                            report.synapses_reinforced += 1;
                            report.details.push(format!("💚 [IA CONSOLIDOU & REFORÇOU]: {} -> {}", subj, prop));
                        } else {
                            report.details.push(format!("🧠 [IA CONSOLIDOU FATO]: {} -> {}", subj, prop));
                        }
                    }
                }
            }

            // 3. Process inhibitory rules and safeguards created by AI
            if let Some(rules) = val.get("rules_and_safeguards").and_then(|v| v.as_array()) {
                for r in rules {
                    let raw_rule = r.get("rule").and_then(|v| v.as_str()).unwrap_or("").trim();
                    let reason = r.get("reason").and_then(|v| v.as_str()).unwrap_or("").trim();
                    let rule_text = Self::sanitize_markdown_text(raw_rule);

                    if !rule_text.is_empty() && rule_text.len() >= 3 {
                        let label = format!("Regra: {}", rule_text);
                        let rule_id = self.add_node_with_valence(NodeType::RuleOrAlert, &label, -1);
                        report.rules_created += 1;
                        report.negative_count += 1;

                        if let Some(atena_node) = self.find_node_by_label("Atena") {
                            let _ = self.add_edge(atena_node.id, rule_id, RelationType::AvoidAction);
                        }

                        report.details.push(format!("⚠️ [IA CRIOU REGRA DE PROTEÇÃO]: {} ({})", rule_text, reason));
                    }
                }
            }
        } else {
            report.details.push("⚠️ Formato de resposta da IA não estruturado, aplicando rotina neural padrão...".to_string());
            let fallback = self.run_sleep_cycle();
            return fallback;
        }

        // 4. Poda Sináptica Ativa (Decaimento e remoção de arestas fracas)
        self.apply_decay(0.05);
        report.synapses_pruned = self.prune_weak_synapses(0.20);
        report.details.push(format!("✂️ [PODA SINÁPTICA]: {} arestas em desuso foram podadas", report.synapses_pruned));

        // 5. Save modular brain and clear consolidated wakefulness buffer
        let _ = self.auto_persist_default();
        let _ = Self::clear_vigilia_buffer();

        report
    }

    /// Saves modular partitioned brain lobes and manifest to disk
    pub fn auto_persist_default(&self) -> Result<usize, String> {
        let root = Self::ensure_brain_directories()?;

        let manifest_path = root.join("brain_manifest.atena");


        // 1. Partition nodes and edges by brain lobe
        let mut temporal_engine = MemoryGraphEngine::new(64);
        let mut parietal_engine = MemoryGraphEngine::new(64);
        let mut prefrontal_engine = MemoryGraphEngine::new(64);
        let mut occipital_engine = MemoryGraphEngine::new(64);

        for node in self.nodes.values() {
            let lobe = Self::classify_node_lobe(node);
            let target_engine = match lobe {
                BrainLobe::Hipocampo | BrainLobe::Temporal => &mut temporal_engine,
                BrainLobe::Parietal => &mut parietal_engine,
                BrainLobe::Prefrontal => &mut prefrontal_engine,
                BrainLobe::Occipital => &mut occipital_engine,
            };
            target_engine.nodes.insert(node.id, node.clone());
            target_engine.label_index.insert(node.label.to_lowercase(), node.id);
        }

        // Distribute edges according to source node lobe
        for (src_id, edges) in &self.adjacency {
            if let Some(src_node) = self.nodes.get(src_id) {
                let lobe = Self::classify_node_lobe(src_node);
                let target_engine = match lobe {
                    BrainLobe::Hipocampo | BrainLobe::Temporal => &mut temporal_engine,
                    BrainLobe::Parietal => &mut parietal_engine,
                    BrainLobe::Prefrontal => &mut prefrontal_engine,
                    BrainLobe::Occipital => &mut occipital_engine,
                };
                target_engine.adjacency.insert(*src_id, edges.clone());
            }
        }

        // 2. Save binary lobe files
        let mut total_bytes = 0;
        let p_temporal = root.join("temporal").join("identidades.atena");
        let p_parietal = root.join("parietal").join("associacoes_locais.atena");
        let p_prefrontal = root.join("prefrontal").join("regras_preferencias.atena");
        let p_occipital = root.join("occipital").join("formas_visuals.atena");

        total_bytes += temporal_engine.save_to_compressed_binary(&p_temporal, CompressionType::Zstd)?;
        total_bytes += parietal_engine.save_to_compressed_binary(&p_parietal, CompressionType::Zstd)?;
        total_bytes += prefrontal_engine.save_to_compressed_binary(&p_prefrontal, CompressionType::Zstd)?;
        total_bytes += occipital_engine.save_to_compressed_binary(&p_occipital, CompressionType::Zstd)?;

        // 3. Save brain_manifest.atena (Unified snapshot and index)
        total_bytes += self.save_to_compressed_binary(&manifest_path, CompressionType::Zstd)?;

        // 4. Maintain mirror of legacy ~/.atena/memory.atena file
        let legacy_path = Self::default_memory_file_path();
        let _ = self.save_to_compressed_binary(&legacy_path, CompressionType::Zstd);

        Ok(total_bytes)
    }

    /// Loads memory from modular brain manifest (~/.atena/brain/brain_manifest.atena) or legacy file
    pub fn load_default_or_init() -> Self {
        let root = Self::brain_root_path();
        let manifest_path = root.join("brain_manifest.atena");

        // 1. Try loading from modular brain manifest
        if manifest_path.exists() {
            match Self::load_from_compressed_binary(&manifest_path, 128) {
                Ok(mut engine) => {
                    engine.scrub_corrupted_nodes();
                    log::info!(
                        "Modular brain loaded successfully from {}: {} nodes, {} edges",
                        manifest_path.display(),
                        engine.node_count(),
                        engine.edge_count()
                    );
                    return engine;
                }
                Err(err) => {
                    log::warn!(
                        "Failed to load brain_manifest.atena: {}",
                        err
                    );
                }
            }
        }

        // 2. If manifest was missing or contained 0 nodes, try loading individual lobe files
        let mut unified_nodes = HashMap::new();
        let mut unified_label_index = HashMap::new();
        let mut unified_adj = HashMap::new();
        let mut max_id = 0;
        let mut loaded_any = false;

        let lobe_paths = [
            root.join("temporal").join("identidades.atena"),
            root.join("parietal").join("associacoes_locais.atena"),
            root.join("prefrontal").join("regras_preferencias.atena"),
            root.join("occipital").join("formas_visuals.atena"),
        ];

        for path in &lobe_paths {
            if path.exists() {
                if let Ok(lobe_eng) = Self::load_from_compressed_binary(path, 64) {
                    if lobe_eng.node_count() > 0 {
                        loaded_any = true;
                        for (id, node) in lobe_eng.nodes {
                            if id > max_id { max_id = id; }
                            unified_label_index.insert(node.label.to_lowercase(), id);
                            unified_nodes.insert(id, node);
                        }
                        for (src_id, edges) in lobe_eng.adjacency {
                            unified_adj.entry(src_id).or_insert_with(Vec::new).extend(edges);
                        }
                    }
                }
            }
        }

        if loaded_any && !unified_nodes.is_empty() {
            let mut engine = Self {
                nodes: unified_nodes,
                label_index: unified_label_index,
                adjacency: unified_adj,
                next_id: max_id + 1,
                hot_cache: HotCache::new(128),
            };
            engine.scrub_corrupted_nodes();
            let _ = engine.auto_persist_default();
            return engine;
        }

        // 3. Fallback: load legacy ~/.atena/memory.atena file and migrate
        let legacy_path = Self::default_memory_file_path();
        if legacy_path.exists() {
            if let Ok(mut legacy_engine) = Self::load_from_compressed_binary(&legacy_path, 128) {
                if legacy_engine.node_count() > 0 {
                    log::info!("Migrating legacy memory file to modular topology in ~/.atena/brain/...");
                    legacy_engine.scrub_corrupted_nodes();
                    let _ = legacy_engine.auto_persist_default();
                    return legacy_engine;
                }
            }
        }

        // 4. Resilience Fallback: Reconstruct memory graph from Markdown episodic chain
        let mut engine = Self::with_default_cache();
        if engine.rebuild_from_episodes_if_empty() {
            log::info!(
                "Memory graph restored successfully from episodic journals: {} nodes, {} edges",
                engine.node_count(),
                engine.edge_count()
            );
            return engine;
        }

        engine
    }

    // =========================================================================
    // Active Retrieval & Chat Context Injection
    // =========================================================================

    /// Bilingual (PT/EN) stop-words evaluated zero-alloc to filter search noise
    #[inline]
    pub fn is_stop_word(word: &str) -> bool {
        matches!(
            word,
            "the" | "be" | "to" | "of" | "and" | "a" | "in" | "that" | "have" | "it" | "for"
                | "not" | "on" | "with" | "he" | "as" | "you" | "do" | "at" | "this" | "but"
                | "his" | "by" | "from" | "they" | "we" | "say" | "her" | "she" | "or" | "an"
                | "will" | "my" | "one" | "all" | "would" | "there" | "their" | "what" | "so"
                | "up" | "out" | "if" | "about" | "who" | "get" | "which" | "go" | "me" | "when"
                | "make" | "can" | "like" | "time" | "no" | "just" | "him" | "know" | "take"
                | "into" | "your" | "good" | "some" | "could" | "them" | "see" | "other" | "than"
                | "then" | "now" | "look" | "only" | "come" | "its" | "over" | "think" | "also"
                | "back" | "after" | "use" | "two" | "how" | "our" | "work" | "first" | "well"
                | "way" | "even" | "new" | "want" | "because" | "any" | "these" | "give" | "day"
                | "most" | "us"
                | "de" | "que" | "da" | "em" | "um" | "para" | "com" | "nao" | "não"
                | "uma" | "os" | "se" | "na" | "por" | "mais" | "dos" | "como"
                | "mas" | "foi" | "ao" | "ele" | "das" | "tem" | "qual" | "seu" | "sua" | "ou"
                | "ser" | "quando" | "muito" | "nos" | "já" | "ja" | "eu" | "também" | "tambem"
                | "só" | "pelo" | "pela" | "ate" | "até" | "isso" | "ela" | "entre"
                | "era" | "depois" | "sem" | "mesmo" | "aos" | "ter" | "seus" | "quem" | "nas"
                | "meu" | "esse" | "eles" | "estão" | "estao" | "você" | "voce" | "tinha"
                | "foram" | "essa" | "num" | "nem" | "suas" | "minha" | "meus" | "minhas"
                | "dele" | "dela" | "deles" | "delas" | "este" | "esta" | "estes" | "estas"
                | "onde" | "aonde" | "qualquer" | "coisa" | "algo"
        )
    }

    /// Searches query keywords in graph and returns associative context
    pub fn search_active_context_for_query(&mut self, query: &str) -> Option<String> {
        if self.nodes.is_empty() {
            return None;
        }

        let query_lower = query.to_lowercase();
        let mut matched_labels: Vec<String> = Vec::new();

        // 1. Procurar correspondência exata ou por palavra filtrada de stop words
        let words: Vec<&str> = query_lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() >= 3 && !Self::is_stop_word(w))
            .collect();

        for (label, _) in &self.label_index {
            let lbl_lower = label.to_lowercase();
            // Verifica se a query contém o label ou se alguma palavra significativa bate
            if query_lower.contains(&lbl_lower)
                || words.iter().any(|w| *w == lbl_lower || (lbl_lower.len() >= 4 && lbl_lower.contains(*w)))
            {
                if !matched_labels.contains(label) {
                    matched_labels.push(label.clone());
                }
            }
        }

        // 1.1 Self-reference / user identity detection (e.g. "who am i", "what is my name", "quem sou eu")
        let self_referential = [
            "quem sou eu",
            "quem eu sou",
            "meu nome",
            "me conhece",
            "sabe quem sou",
            "sabe meu nome",
            "sobre mim",
            "me chamo",
            "quem sou",
            "who am i",
            "who i am",
            "my name",
            "do you know me",
            "know my name",
            "about me",
            "know who i am",
        ];
        if self_referential.iter().any(|p| query_lower.contains(p)) {
            for (label, _) in &self.label_index {
                let lbl_lower = label.to_lowercase();
                if lbl_lower == "user" || lbl_lower == "usuário" || lbl_lower == "usuario" {
                    if !matched_labels.contains(label) {
                        matched_labels.push(label.clone());
                    }
                }
            }
        }

        if matched_labels.is_empty() {
            return None;
        }

        // 2. For each matched label, perform Spreading Activation and reinforce
        let mut all_paths: Vec<AssociationPath> = Vec::new();
        for label in &matched_labels {
            if let Ok(paths) = self.traverse_associations(label, 3) {
                for p in paths {
                    all_paths.push(p);
                }
            }
        }

        if all_paths.is_empty() {
            return None;
        }

        // Ordenar caminhos por peso
        all_paths.sort_by(|a, b| {
            b.total_weight
                .partial_cmp(&a.total_weight)
                .unwrap_or(Ordering::Equal)
        });
        all_paths.truncate(8); // Limitar aos 8 caminhos mais fortes para não estourar tokens

        Some(Self::build_llm_context(&all_paths))
    }

    /// Returns associations linked to the User entity ("User", "Usuario")
    /// to inject the persistent user profile into system prompts.
    pub fn get_user_profile_context(&mut self) -> Option<String> {
        if self.nodes.is_empty() {
            return None;
        }

        let user_aliases = ["usuário", "usuario", "user"];
        let mut user_labels = Vec::new();

        for alias in &user_aliases {
            for (label, _) in &self.label_index {
                if label.to_lowercase() == *alias {
                    if !user_labels.contains(label) {
                        user_labels.push(label.clone());
                    }
                }
            }
        }

        if user_labels.is_empty() {
            return None;
        }

        let mut all_paths: Vec<AssociationPath> = Vec::new();
        for label in &user_labels {
            if let Ok(paths) = self.traverse_associations(label, 2) {
                for p in paths {
                    all_paths.push(p);
                }
            }
        }

        if all_paths.is_empty() {
            return None;
        }

        all_paths.sort_by(|a, b| {
            b.total_weight
                .partial_cmp(&a.total_weight)
                .unwrap_or(Ordering::Equal)
        });
        all_paths.truncate(8);

        Some(Self::build_llm_context(&all_paths))
    }

    /// Retorna um resumo formatado dos registros mais recentes e ativos no grafo cerebral.
    /// Permite que a IA veja o estado real atual da sua memória para evitar contradições,
    /// duplicidades ou dados errados.
    pub fn get_recent_memory_records(&self, limit: usize) -> Option<String> {
        if self.nodes.is_empty() {
            return None;
        }

        let mut all_edges: Vec<(u32, &MemoryEdge)> = Vec::new();
        for (source_id, edges) in &self.adjacency {
            for edge in edges {
                all_edges.push((*source_id, edge));
            }
        }

        if all_edges.is_empty() {
            return None;
        }

        all_edges.sort_by(|a, b| b.1.last_accessed.cmp(&a.1.last_accessed));

        let now = Self::current_timestamp();
        let mut lines = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for (source_id, edge) in all_edges.into_iter().take(limit * 2) {
            if let (Some(source_node), Some(target_node)) = (self.nodes.get(&source_id), self.nodes.get(&edge.target_id)) {
                let key = (source_node.label.to_lowercase(), target_node.label.to_lowercase());
                if seen.insert(key) {
                    let ts = if edge.created_at > 0 { edge.created_at } else { edge.last_accessed };
                    let time_str = Self::format_memory_timestamp(ts, now);
                    lines.push(format!("- {} -> {} [saved at: {}]", source_node.label, target_node.label, time_str));
                    if lines.len() >= limit {
                        break;
                    }
                }
            }
        }

        if lines.is_empty() {
            None
        } else {
            Some(lines.join("\n"))
        }
    }

    // =========================================================================
    // Runtime Autonomous Cognitive Memory
    // =========================================================================

    /// Normaliza aspas tipográficas (smart quotes) para aspas ASCII simples
    pub fn normalize_smart_quotes(text: &str) -> String {
        text.replace(['“', '”', '„', '‟', '«', '»'], "\"")
            .replace(['‘', '’', '‚', '‛'], "'")
    }

    /// Extrai um atributo com segurança de uma tag normalizada (ex: sujeito="...", propriedade="...")
    pub fn extract_attribute_safe(tag_str: &str, attr_names: &[&str]) -> Option<String> {
        let tag_normalized = Self::normalize_smart_quotes(tag_str);

        for name in attr_names {
            let mut search_from = 0;
            while let Some(pos) = tag_normalized[search_from..].find(name) {
                let actual_pos = search_from + pos;
                search_from = actual_pos + name.len();

                // Garante que é uma palavra inteira (precedida por espaço, '<', etc.)
                if actual_pos > 0 {
                    let prev = tag_normalized[..actual_pos].chars().last().unwrap_or(' ');
                    if !prev.is_whitespace() && prev != '<' {
                        continue;
                    }
                }

                let rest = &tag_normalized[actual_pos + name.len()..];
                let rest_trimmed = rest.trim_start();
                if !rest_trimmed.starts_with('=') {
                    continue;
                }

                let after_eq = rest_trimmed[1..].trim_start();
                if after_eq.is_empty() {
                    continue;
                }

                if after_eq.starts_with('"') {
                    let inner = &after_eq[1..];
                    if let Some(end_quote) = inner.find('"') {
                        let val = inner[..end_quote].trim();
                        if !val.is_empty() {
                            return Some(val.to_string());
                        }
                    }
                } else if after_eq.starts_with('\'') {
                    let inner = &after_eq[1..];
                    if let Some(end_quote) = inner.find('\'') {
                        let val = inner[..end_quote].trim();
                        if !val.is_empty() {
                            return Some(val.to_string());
                        }
                    }
                } else {
                    let end_pos = after_eq
                        .find(|c: char| c.is_whitespace() || c == '/' || c == '>')
                        .unwrap_or(after_eq.len());
                    let val = after_eq[..end_pos].trim();
                    if !val.is_empty() {
                        return Some(val.to_string());
                    }
                }
            }
        }
        None
    }

    /// Removes invalid or corrupted nodes generated by malformed tags
    pub fn scrub_corrupted_nodes(&mut self) {
        let bad_node_ids: Vec<u32> = self
            .nodes
            .iter()
            .filter(|(_, n)| {
                n.label.contains("<memoria")
                    || n.label.contains("<memory")
                    || n.label.contains("<habilidade")
                    || n.label.contains("<skill")
                    || n.label.contains("propriedade=")
                    || n.label.contains("sujeito=")
                    || n.label.contains("tipo=")
                    || n.label.contains("valencia=")
                    || (n.type_flag == NodeType::RuleOrAlert && (n.label.to_lowercase().contains("wedding") || n.label.to_lowercase().contains("casamento") || n.label.to_lowercase().contains("aniversário")))
                    || n.label.len() > 100
            })
            .map(|(id, _)| *id)
            .collect();

        if !bad_node_ids.is_empty() {
            log::info!("Scrubbing {} corrupted nodes from memory graph", bad_node_ids.len());
            for id in bad_node_ids {
                let _ = self.delete_node(id);
            }
            let _ = self.auto_persist_default();
        }
    }

    /// Inserts or updates a declarative fact in the associative memory graph
    pub fn insert_declarative_fact(
        &mut self,
        subj: &str,
        prop: &str,
        valence: i8,
        raw_session: Option<&str>,
        memory_ts: u64,
    ) {
        let subj_id = self.get_or_create_node_with_valence_and_session(NodeType::Object, subj, 0, raw_session);
        let attr_id = self.get_or_create_node_with_valence_and_session(NodeType::Attribute, prop, valence, raw_session);
        let _ = self.add_edge_with_timestamp(subj_id, attr_id, RelationType::HasProperty, memory_ts);

        // If it's a "Key: Value" property and subject already has a property with the same "Key:",
        // replace the old edge to keep memory up to date without contradictions
        if let Some((key, _)) = prop.split_once(':') {
            let key_trimmed = key.trim();
            let key_lower = key_trimmed.to_lowercase();
            let is_multi_relation = key_lower == "amigo" || key_lower == "amiga" || key_lower == "friend"
                || key_lower == "colega" || key_lower == "colleague" || key_lower == "filho" || key_lower == "filha";

            if key_trimmed.len() >= 3 && !is_multi_relation {
                let key_prefix = format!("{}:", key_lower);
                if let Some(edges) = self.adjacency.get_mut(&subj_id) {
                    edges.retain(|e| {
                        if e.target_id != attr_id && e.relation_type == RelationType::HasProperty {
                            if let Some(target_node) = self.nodes.get(&e.target_id) {
                                return !target_node.label.to_lowercase().starts_with(&key_prefix);
                            }
                        }
                        true
                    });
                }
            }
        }

        // Automatic detection and replacement of events with conflicting dates
        let prop_lower = prop.to_lowercase();
        let event_keywords = ["casamento", "wedding", "aniversário", "aniversario", "birthday", "noivado", "festa", "viagem", "reunião", "meeting"];
        let has_date_pattern = |s: &str| -> bool {
            let b = s.as_bytes();
            for i in 0..b.len().saturating_sub(4) {
                if b[i].is_ascii_digit() && b[i+1].is_ascii_digit() && (b[i+2] == b'/' || b[i+2] == b'-') && b[i+3].is_ascii_digit() {
                    return true;
                }
            }
            false
        };

        if has_date_pattern(prop) {
            if let Some(event_kw) = event_keywords.iter().find(|kw| prop_lower.contains(*kw)) {
                let kw = *event_kw;
                let mut pruned_orphans = Vec::new();
                if let Some(edges) = self.adjacency.get_mut(&subj_id) {
                    edges.retain(|e| {
                        if e.target_id != attr_id && e.relation_type == RelationType::HasProperty {
                            if let Some(target_node) = self.nodes.get(&e.target_id) {
                                let t_lower = target_node.label.to_lowercase();
                                if t_lower.contains(kw) && has_date_pattern(&target_node.label) {
                                    log::info!("🔄 [UPDATED RECORD]: Removing obsolete previous date/record: {}", target_node.label);
                                    pruned_orphans.push(e.target_id);
                                    return false;
                                }
                            }
                        }
                        true
                    });
                }
                for orphan_id in pruned_orphans {
                    if !self.has_any_connections(orphan_id) {
                        let _ = self.delete_node(orphan_id);
                    }
                }
            }
        }

        let is_user_subject = subj.to_lowercase() == "usuário" || subj.to_lowercase() == "usuario" || subj.to_lowercase() == "user";
        let is_name_prop = prop.to_lowercase().starts_with("nome:") || prop.to_lowercase().starts_with("nome é") || prop.to_lowercase().starts_with("name:") || prop.to_lowercase().starts_with("name is");

        // 1. Identity and facts for the User themselves
        if is_user_subject || is_name_prop {
            let user_id = self.get_or_create_node(NodeType::Object, "User");
            if subj_id != user_id {
                let _ = self.add_edge_with_timestamp(user_id, subj_id, RelationType::HasProperty, memory_ts);
            }
            let _ = self.add_edge_with_timestamp(user_id, attr_id, RelationType::HasProperty, memory_ts);
            let _ = self.reinforce_edge(user_id, attr_id);
        }

        // 2. Interpersonal relationships when subject is User
        let prop_lower_clean = prop.to_lowercase();
        if is_user_subject {
            let user_id = self.get_or_create_node(NodeType::Object, "User");
            let prefixes = [
                "friend:", "amiga:", "amigo:", "wife:", "esposa:", "husband:", "marido:",
                "colleague:", "colega:", "coworker:", "sister:", "irmã:", "brother:", "irmão:",
                "irma:", "irmao:", "son:", "filho:", "daughter:", "filha:", "father:", "pai:",
                "mother:", "mãe:", "mae:", "partner:", "boss:", "mentor:",
                "amiga de ", "amigo de ", "amiga da ", "amigo do ", "friend of ", "wife of ",
                "husband of ", "sister of ", "brother of ", "esposa de ", "marido de ",
                "irmã de ", "irmão de ", "filho de ", "filha de ", "colega de "
            ];
            for pfx in prefixes {
                if prop_lower_clean.starts_with(pfx) {
                    let person_name = prop[pfx.len()..].trim();
                    if !person_name.is_empty() {
                        let person_id = self.get_or_create_node(NodeType::Object, person_name);
                        let _ = self.add_edge_with_timestamp(user_id, person_id, RelationType::HasProperty, memory_ts);
                        let _ = self.add_edge_with_timestamp(person_id, user_id, RelationType::HasProperty, memory_ts);
                        let _ = self.reinforce_edge(user_id, person_id);
                    }
                    break;
                }
            }
        }

        // 3. Interpersonal relationships when subject is the person
        if !is_user_subject {
            let relationship_indicators = [
                "friend of user", "friend of the user", "user's friend",
                "wife of user", "wife of the user", "user's wife",
                "husband of user", "husband of the user", "user's husband",
                "colleague of user", "colleague of the user", "user's colleague",
                "coworker of user", "coworker of the user", "user's coworker",
                "sister of user", "sister of the user", "user's sister",
                "brother of user", "brother of the user", "user's brother",
                "son of user", "son of the user", "user's son",
                "daughter of user", "daughter of the user", "user's daughter",
                "father of user", "father of the user", "user's father",
                "mother of user", "mother of the user", "user's mother",
                "partner of user", "partner of the user", "user's partner",
                "amiga do usuário", "amigo do usuário", "amiga do usuario", "amigo do usuario",
                "esposa do usuário", "marido do usuário", "esposa do usuario", "marido do usuario",
                "irmã do usuário", "irmão do usuário", "irma do usuario", "irmao do usuario",
                "colega do usuário", "colega do usuario", "familiar do usuário", "familiar do usuario"
            ];
            if relationship_indicators.iter().any(|ind| prop_lower_clean.contains(ind)) {
                let user_id = self.get_or_create_node(NodeType::Object, "User");
                let _ = self.add_edge_with_timestamp(user_id, subj_id, RelationType::HasProperty, memory_ts);
                let _ = self.add_edge_with_timestamp(subj_id, user_id, RelationType::HasProperty, memory_ts);
                let _ = self.reinforce_edge(user_id, subj_id);
            }
        }

        if valence > 0 {
            let _ = self.reinforce_edge(subj_id, attr_id);
        }
    }

    /// Inserts an inhibitory rule fact into the associative memory graph
    pub fn insert_rule_fact(
        &mut self,
        subj: &str,
        prop: &str,
        raw_session: Option<&str>,
        memory_ts: u64,
    ) {
        let subj_id = self.get_or_create_node_with_valence_and_session(NodeType::Object, subj, 0, raw_session);
        let rule_label = if prop.starts_with("Regra:") || prop.starts_with("Rule:") {
            prop.to_string()
        } else {
            format!("Rule: {}", prop)
        };
        let rule_id = self.add_node_with_valence_and_session(NodeType::RuleOrAlert, &rule_label, -1, raw_session.map(|s| s.to_string()));
        let _ = self.add_edge_with_timestamp(subj_id, rule_id, RelationType::AvoidAction, memory_ts);
        if let Some(atena_node) = self.find_node_by_label("Atena") {
            let _ = self.add_edge_with_timestamp(atena_node.id, rule_id, RelationType::AvoidAction, memory_ts);
        }
    }

    /// Inserts a container/location fact into the associative memory graph
    pub fn insert_container_fact(
        &mut self,
        subj: &str,
        prop: &str,
        raw_session: Option<&str>,
        memory_ts: u64,
    ) {
        let subj_id = self.get_or_create_node_with_valence_and_session(NodeType::Object, subj, 0, raw_session);
        let loc_id = self.get_or_create_node_with_valence_and_session(NodeType::Container, prop, 0, raw_session);
        let _ = self.add_edge_with_timestamp(subj_id, loc_id, RelationType::LocatedIn, memory_ts);
    }

    /// Reconstructs the associative memory graph from Markdown episodic chain files if the in-memory graph is empty
    pub fn rebuild_from_episodes_if_empty(&mut self) -> bool {
        if !self.nodes.is_empty() {
            return false;
        }

        let dir = Self::episodios_dir();
        if !dir.exists() {
            return false;
        }

        let mut entries = match std::fs::read_dir(&dir) {
            Ok(rd) => rd
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| {
                    p.extension().and_then(|s| s.to_str()) == Some("md")
                        && p.file_name()
                            .and_then(|s| s.to_str())
                            .map(|s| s.starts_with("episodio_"))
                            .unwrap_or(false)
                })
                .collect::<Vec<_>>(),
            Err(_) => return false,
        };

        if entries.is_empty() {
            return false;
        }

        entries.sort();

        log::info!("Rebuilding memory graph from {} episodic journal files...", entries.len());
        let mut restored_facts = 0;

        for path in entries {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let mut ep_ts = Self::current_timestamp();
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("- **ID do Episódio**:") {
                        if let Some(id_part) = trimmed.split('`').nth(1) {
                            let parts: Vec<&str> = id_part.split('_').collect();
                            if parts.len() >= 2 {
                                if let Ok(ts) = parts[1].parse::<u64>() {
                                    ep_ts = ts;
                                }
                            }
                        }
                    }

                    if trimmed.starts_with("- 💚 [MEMÓRIA SALVA & REFORÇADA]:")
                        || trimmed.starts_with("- 💚 [SAVED & REINFORCED MEMORY]:")
                        || trimmed.starts_with("- 🧠 [MEMÓRIA SALVA]:")
                        || trimmed.starts_with("- 🧠 [SAVED MEMORY]:")
                        || trimmed.starts_with("- [MEMÓRIA SALVA & REFORÇADA]:")
                        || trimmed.starts_with("- [SAVED & REINFORCED MEMORY]:")
                        || trimmed.starts_with("- [MEMÓRIA SALVA]:")
                        || trimmed.starts_with("- [SAVED MEMORY]:")
                    {
                        let clean = trimmed
                            .trim_start_matches('-')
                            .trim()
                            .replace("[SAVED & REINFORCED MEMORY]:", "")
                            .replace("[MEMÓRIA SALVA & REFORÇADA]:", "")
                            .replace("[SAVED MEMORY]:", "")
                            .replace("[MEMÓRIA SALVA]:", "")
                            .replace("💚", "")
                            .replace("🧠", "");
                        if let Some((subj, prop)) = clean.split_once("->") {
                            let s = subj.trim();
                            let p = prop.trim();
                            if !s.is_empty() && !p.is_empty() {
                                self.insert_declarative_fact(s, p, 1, None, ep_ts);
                                restored_facts += 1;
                            }
                        }
                    } else if trimmed.starts_with("- ⚠️ [REGRA INIBITÓRIA SALVA]:")
                        || trimmed.starts_with("- ⚠️ [SAVED INHIBITORY RULE]:")
                        || trimmed.starts_with("- [SAVED INHIBITORY RULE]:")
                        || trimmed.starts_with("- [REGRA INIBITÓRIA SALVA]:")
                    {
                        let clean = trimmed
                            .trim_start_matches('-')
                            .trim()
                            .replace("[SAVED INHIBITORY RULE]:", "")
                            .replace("[REGRA INIBITÓRIA SALVA]:", "")
                            .replace("⚠️", "");
                        if let Some((subj, prop)) = clean.split_once("->") {
                            let s = subj.trim();
                            let p = prop.trim();
                            if !s.is_empty() && !p.is_empty() {
                                self.insert_rule_fact(s, p, None, ep_ts);
                                restored_facts += 1;
                            }
                        }
                    } else if trimmed.starts_with("- 📍 [LOCALIZAÇÃO SALVA]:")
                        || trimmed.starts_with("- 📍 [SAVED LOCATION]:")
                        || trimmed.starts_with("- [SAVED LOCATION]:")
                        || trimmed.starts_with("- [LOCALIZAÇÃO SALVA]:")
                    {
                        let clean = trimmed
                            .trim_start_matches('-')
                            .trim()
                            .replace("[SAVED LOCATION]:", "")
                            .replace("[LOCALIZAÇÃO SALVA]:", "")
                            .replace("📍", "");
                        let split_res = clean.split_once(" in ").or_else(|| clean.split_once(" em "));
                        if let Some((subj, loc)) = split_res {
                            let s = subj.trim();
                            let l = loc.trim();
                            if !s.is_empty() && !l.is_empty() {
                                self.insert_container_fact(s, l, None, ep_ts);
                                restored_facts += 1;
                            }
                        }
                    }
                }
            }
        }

        if restored_facts > 0 {
            log::info!(
                "Successfully restored {} facts into memory graph ({} nodes, {} edges)",
                restored_facts,
                self.node_count(),
                self.edge_count()
            );
            let _ = self.auto_persist_default();
            true
        } else {
            false
        }
    }

    /// Resolves relative temporal terms (e.g., "Tomorrow", "Amanhã", "Yesterday", "Ontem", "Today", "Hoje")
    /// into absolute dates formatted as DD/MM/YYYY.
    pub fn resolve_relative_temporal_date(prop: &str) -> String {
        let now = chrono::Local::now();
        let tomorrow_str = (now + chrono::Duration::days(1)).format("%d/%m/%Y").to_string();
        let today_str = now.format("%d/%m/%Y").to_string();
        let yesterday_str = (now - chrono::Duration::days(1)).format("%d/%m/%Y").to_string();

        let mut resolved = prop.to_string();
        for token in &["Tomorrow", "tomorrow", "TOMORROW", "Amanhã", "amanhã", "Amanha", "amanha"] {
            if resolved.contains(token) {
                resolved = resolved.replace(token, &tomorrow_str);
            }
        }
        for token in &["Yesterday", "yesterday", "YESTERDAY", "Ontem", "ontem"] {
            if resolved.contains(token) {
                resolved = resolved.replace(token, &yesterday_str);
            }
        }
        for token in &["Today", "today", "TODAY", "Hoje", "hoje"] {
            if resolved.contains(token) {
                resolved = resolved.replace(token, &today_str);
            }
        }
        resolved
    }

    /// Extracts and processes autonomous memory tags emitted by the AI during conversation.
    /// Returns a tuple: (clean_text_without_tags, list_of_memorized_facts).
    pub fn extract_and_apply_memory_tags(&mut self, raw_text: &str) -> (String, Vec<String>) {
        let mut learned = Vec::new();
        let normalized = Self::normalize_smart_quotes(raw_text);

        // 1. Extract Forget / Memory Correction tags (<forget ... />, <esquecer ... />, <remover ... />)
        let forget_prefixes = ["<forget", "<esquecer", "<remover"];
        for prefix in &forget_prefixes {
            let mut search_pos = 0;
            while let Some(start_offset) = normalized[search_pos..].find(prefix) {
                let start = search_pos + start_offset;
                let rest_from_start = &normalized[start..];

                let tag_end_offset = if let Some(close_pos) = rest_from_start.find("/>") {
                    close_pos + 2
                } else if let Some(close_pos) = rest_from_start.find('>') {
                    close_pos + 1
                } else {
                    rest_from_start.len()
                };

                let tag_chunk = &rest_from_start[..tag_end_offset.min(500)];
                search_pos = start + tag_chunk.len().max(prefix.len());

                let raw_subj = Self::extract_attribute_safe(tag_chunk, &["sujeito", "subject", "entidade", "entity"]).unwrap_or_default();
                let raw_prop = Self::extract_attribute_safe(tag_chunk, &["propriedade", "property", "fato", "fact", "alvo", "target"]).unwrap_or_default();

                let subj = Self::sanitize_markdown_text(&raw_subj);
                let prop = Self::sanitize_markdown_text(&raw_prop);

                self.apply_forget_rule(&subj, &prop, &mut learned);
            }
        }

        // 2. Extract Declarative Memory tags (<memory ... />, <memoria ... />)
        let mem_prefixes = ["<memoria", "<memory", "<memorizar"];
        for prefix in &mem_prefixes {
            let mut search_pos = 0;
            while let Some(start_offset) = normalized[search_pos..].find(prefix) {
                let start = search_pos + start_offset;
                let rest_from_start = &normalized[start..];

                let tag_end_offset = if let Some(close_pos) = rest_from_start.find("/>") {
                    close_pos + 2
                } else if let Some(close_pos) = rest_from_start.find('>') {
                    close_pos + 1
                } else {
                    rest_from_start.len()
                };

                let tag_chunk = &rest_from_start[..tag_end_offset.min(500)];
                search_pos = start + tag_chunk.len().max(prefix.len());

                let raw_subj = Self::extract_attribute_safe(tag_chunk, &["sujeito", "subject", "entidade", "entity"]).unwrap_or_default();
                let raw_prop = Self::extract_attribute_safe(tag_chunk, &["propriedade", "property", "fato", "fact"]).unwrap_or_default();
                let raw_valence = Self::extract_attribute_safe(tag_chunk, &["valencia", "valence", "peso", "weight"]).unwrap_or_default();
                let raw_type = Self::extract_attribute_safe(tag_chunk, &["tipo", "type", "classe", "class"]).unwrap_or_default();
                let raw_session = Self::extract_attribute_safe(tag_chunk, &["session", "sessao", "conversa", "session_id"]);

                let subj = Self::sanitize_markdown_text(&raw_subj);
                let prop = Self::resolve_relative_temporal_date(&Self::sanitize_markdown_text(&raw_prop));

                if !subj.is_empty() && !prop.is_empty() {
                    let mut valence: i8 = raw_valence.parse().unwrap_or(0);
                    let mut node_type = match raw_type.to_lowercase().as_str() {
                        "container" | "local" | "localizacao" => NodeType::Container,
                        "ruleoralert" | "regra" | "regrainibitoria" | "alerta" | "inhibition" => NodeType::RuleOrAlert,
                        _ => NodeType::Attribute,
                    };

                    // Guardrail: Inhibitory rules always have negative valence (-1)
                    if node_type == NodeType::RuleOrAlert && valence >= 0 {
                        valence = -1;
                    }

                    if valence < 0 {
                        node_type = NodeType::RuleOrAlert;
                    }

                    // Positive events / milestones guardrail: never treat weddings, birthdays as negative
                    let is_event_or_positive = {
                        let combined = format!("{} {}", subj, prop).to_lowercase();
                        combined.contains("casamento")
                            || combined.contains("wedding")
                            || combined.contains("aniversário")
                            || combined.contains("aniversario")
                            || combined.contains("birthday")
                            || combined.contains("evento")
                            || combined.contains("festa")
                            || combined.contains("noivado")
                            || combined.contains("data")
                            || combined.contains("date:")
                    };

                    if node_type == NodeType::RuleOrAlert && is_event_or_positive {
                        node_type = NodeType::Attribute;
                        if valence < 1 {
                            valence = 1;
                        }
                    }

                    let memory_ts = Self::current_timestamp();

                    if node_type == NodeType::RuleOrAlert || valence == -1 {
                        let rule_label = if prop.starts_with("Regra:") || prop.starts_with("Rule:") {
                            prop.clone()
                        } else {
                            format!("Rule: {}", prop)
                        };
                        self.insert_rule_fact(&subj, &rule_label, raw_session.as_deref(), memory_ts);
                        learned.push(format!("⚠️ [SAVED INHIBITORY RULE]: {} -> {}", subj, rule_label));
                    } else if node_type == NodeType::Container {
                        self.insert_container_fact(&subj, &prop, raw_session.as_deref(), memory_ts);
                        learned.push(format!("📍 [SAVED LOCATION]: {} in {}", subj, prop));
                    } else {
                        self.insert_declarative_fact(&subj, &prop, valence, raw_session.as_deref(), memory_ts);
                        if valence > 0 {
                            learned.push(format!("💚 [SAVED & REINFORCED MEMORY]: {} -> {}", subj, prop));
                        } else {
                            learned.push(format!("🧠 [SAVED MEMORY]: {} -> {}", subj, prop));
                        }
                    }
                }
            }
        }

        // 3. Extract Procedural Skill tags (<skill ... />, <habilidade ... />)
        let skill_prefixes = ["<habilidade", "<skill"];
        for prefix in &skill_prefixes {
            let mut search_pos = 0;
            while let Some(start_offset) = normalized[search_pos..].find(prefix) {
                let start = search_pos + start_offset;
                let rest_from_start = &normalized[start..];

                let tag_end_offset = if let Some(close_pos) = rest_from_start.find("/>") {
                    close_pos + 2
                } else if let Some(close_pos) = rest_from_start.find('>') {
                    close_pos + 1
                } else {
                    rest_from_start.len()
                };

                let tag_chunk = &rest_from_start[..tag_end_offset.min(800)];
                search_pos = start + tag_chunk.len().max(prefix.len());

                let raw_name = Self::extract_attribute_safe(tag_chunk, &["nome", "name", "titulo", "title"]).unwrap_or_default();
                let raw_desc = Self::extract_attribute_safe(tag_chunk, &["descricao", "description", "desc"]).unwrap_or_default();
                let raw_triggers = Self::extract_attribute_safe(tag_chunk, &["gatilhos", "triggers", "palavras_chave"]).unwrap_or_default();
                let raw_steps = Self::extract_attribute_safe(tag_chunk, &["passos", "steps", "workflow"]).unwrap_or_default();
                let raw_refinement = Self::extract_attribute_safe(tag_chunk, &["refinamento", "refinement", "nota", "ajuste"]).unwrap_or_default();

                if !raw_name.trim().is_empty() && raw_name.len() <= 80 && !raw_name.contains('<') {
                    let triggers_vec: Vec<String> = raw_triggers
                        .split(|c: char| c == ',' || c == ';' || c == '|')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty() && s.len() <= 60)
                        .collect();

                    let steps_vec: Vec<String> = raw_steps
                        .split('|')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty() && s.len() <= 160)
                        .collect();

                    let skill = Self::learn_or_refine_skill(
                        &raw_name,
                        &raw_desc,
                        triggers_vec,
                        steps_vec,
                        if raw_refinement.trim().is_empty() { None } else { Some(raw_refinement) },
                    );

                    learned.push(format!("🛠️ [SKILL LEARNED/REFINED]: {} (v{})", skill.name, skill.version));
                }
            }
        }

        // 4. Full cleanup of all memory/forget tags from returned text
        let mut clean_text = raw_text.to_string();
        for tag_prefix in &["<memoria", "<memory", "<memorizar", "<habilidade", "<skill", "<forget", "<esquecer", "<remover"] {
            while let Some(start) = clean_text.to_lowercase().find(tag_prefix) {
                let slice = &clean_text[start..];
                let remove_len = if let Some(end) = slice.find("/>") {
                    end + 2
                } else if let Some(end) = slice.find('>') {
                    end + 1
                } else {
                    slice.len()
                };
                clean_text.replace_range(start..start + remove_len, "");
            }
        }

        if !learned.is_empty() {
            let _ = self.auto_persist_default();
        }

        (clean_text.trim().to_string(), learned)
    }

    // =========================================================================
    // Episodic Memory: Linked Markdown Turn Chain
    // =========================================================================

    /// Directory path where episodic Markdown turn files are stored
    pub fn episodios_dir() -> PathBuf {
        let root = Self::brain_root_path();
        root.join("episodios")
    }

    /// Control and index file path for episodic chain metadata
    pub fn chain_info_path() -> PathBuf {
        Self::episodios_dir().join("corrente_info.json")
    }

    /// Clears all episodic markdown files and chain metadata from ~/.atena/brain/episodios/
    pub fn clear_episodios() -> Result<(), String> {
        let dir = Self::episodios_dir();
        if dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_file() {
                        let _ = std::fs::remove_file(path);
                    }
                }
            }
        }
        Ok(())
    }

    /// Loads current episodic chain metadata
    pub fn load_chain_info() -> EpisodicChainInfo {
        let path = Self::chain_info_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(info) = serde_json::from_str::<EpisodicChainInfo>(&content) {
                    return info;
                }
            }
        }
        EpisodicChainInfo::default()
    }

    /// Saves updated episodic chain metadata
    pub fn save_chain_info(info: &EpisodicChainInfo) -> Result<(), String> {
        let path = Self::chain_info_path();
        let json = serde_json::to_string_pretty(info).map_err(|e| e.to_string())?;
        std::fs::write(&path, json).map_err(|e| e.to_string())
    }

    /// Records a new conversational turn in the Markdown cognitive episode chain.
    /// Creates the episode file, links to the previous link, and updates the forward link in the previous file.
    pub fn record_episodic_turn(
        user_msg: &str,
        ai_resp: &str,
        learned_memories: &[String],
        session_id: Option<&str>,
        session_title: Option<&str>,
    ) -> Result<String, String> {
        let user_trimmed = user_msg.trim();
        let ai_trimmed = ai_resp.trim();

        if user_trimmed.is_empty() && ai_trimmed.is_empty() {
            return Err("Empty user message and AI response".into());
        }

        let dir = Self::episodios_dir();
        std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create episodes directory: {}", e))?;

        let mut chain_info = Self::load_chain_info();
        let new_index = chain_info.total_episodes + 1;
        let now_epoch = Self::current_timestamp();
        let now_local = chrono::Local::now();
        let datetime_str = now_local.format("%d/%m/%Y %H:%M:%S").to_string();
        let date_file_slug = now_local.format("%Y%m%d_%H%M%S").to_string();

        let file_name = format!("episodio_{}_{:04}.md", date_file_slug, new_index);
        let file_path = dir.join(&file_name);

        let prev_file = chain_info.last_episode_file.clone();

        // If there is a previous link, update the previous file with a forward link to this new episode
        if let Some(ref prev_filename) = prev_file {
            let prev_path = dir.join(prev_filename);
            if prev_path.exists() {
                if let Ok(prev_content) = std::fs::read_to_string(&prev_path) {
                    let placeholders = [
                        "- **Next Link**: (awaiting next link)",
                        "- **Elo Seguinte**: (aguardando próximo elo)",
                    ];
                    let mut updated = prev_content.clone();
                    for placeholder in &placeholders {
                        if updated.contains(placeholder) {
                            updated = updated.replace(
                                placeholder,
                                &format!("- **Next Link**: [Episode #{new_index}](./{})", file_name),
                            );
                        }
                    }
                    if updated != prev_content {
                        let _ = std::fs::write(&prev_path, updated);
                    }
                }
            }
        }

        // Format learned memories block
        let memories_block = if learned_memories.is_empty() {
            "  - No new declarative memories recorded in this turn.".to_string()
        } else {
            learned_memories
                .iter()
                .map(|m| format!("  - {}", m))
                .collect::<Vec<String>>()
                .join("\n")
        };

        let previous_link = if let Some(ref prev) = prev_file {
            format!("[Episode #{prev_index}](./{})", prev, prev_index = new_index - 1)
        } else {
            "Chain Start (First Episode)".to_string()
        };

        let session_id_display = session_id.unwrap_or("").trim();
        let session_title_display = session_title.unwrap_or("").trim();

        let mut session_lines = String::new();
        if !session_id_display.is_empty() {
            session_lines.push_str(&format!("- **Conversation ID**: `{}`\n", session_id_display));
        }
        if !session_title_display.is_empty() {
            session_lines.push_str(&format!("- **Conversation Title**: {}\n", session_title_display));
        }

        let markdown_content = format!(
r#"# Cognitive Episode #{new_index}

- **Episode ID**: `ep_{now_epoch}_{new_index}`
- **Date / Time**: {datetime_str}
{session_lines}- **Previous Link**: {previous_link}
- **Next Link**: (awaiting next link)
- **Memories Learned in This Turn**:
{memories_block}

---

## 👤 User Message
{user_trimmed}

---

## 🤖 Atena Response
{ai_trimmed}
"#,
            new_index = new_index,
            now_epoch = now_epoch,
            datetime_str = datetime_str,
            session_lines = session_lines,
            previous_link = previous_link,
            memories_block = memories_block,
            user_trimmed = user_trimmed,
            ai_trimmed = ai_trimmed
        );

        std::fs::write(&file_path, markdown_content)
            .map_err(|e| format!("Failed to save episode {}: {}", file_path.display(), e))?;

        // Update chain info
        chain_info.total_episodes = new_index;
        if chain_info.first_episode_file.is_none() {
            chain_info.first_episode_file = Some(file_name.clone());
        }
        chain_info.last_episode_file = Some(file_name.clone());
        chain_info.updated_at = now_epoch;
        let _ = Self::save_chain_info(&chain_info);

        Ok(file_name)
    }

    /// Returns a summary of the recent episode chain for prompt context continuity
    pub fn get_recent_episodes_context(limit: usize) -> Option<String> {
        let chain_info = Self::load_chain_info();
        if chain_info.total_episodes == 0 {
            return None;
        }

        let dir = Self::episodios_dir();
        if !dir.exists() {
            return None;
        }

        let mut entries = match std::fs::read_dir(&dir) {
            Ok(rd) => rd
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| {
                    p.extension().and_then(|s| s.to_str()) == Some("md")
                        && p.file_name()
                            .and_then(|s| s.to_str())
                            .map(|s| s.starts_with("episodio_"))
                            .unwrap_or(false)
                })
                .collect::<Vec<PathBuf>>(),
            Err(_) => return None,
        };

        if entries.is_empty() {
            return None;
        }

        entries.sort();

        let mut lines = Vec::new();
        lines.push("[RECENT COGNITIVE EPISODE CHAIN]:".to_string());

        for path in entries.iter().rev().take(limit).rev() {
            if let Ok(content) = std::fs::read_to_string(path) {
                let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("episode.md");
                let mut user_snippet = String::new();
                let mut ai_snippet = String::new();

                let user_part = content.split("## 👤 User Message").nth(1)
                    .or_else(|| content.split("## 👤 Mensagem do Usuário").nth(1));

                if let Some(part) = user_part {
                    if let Some(u_msg) = part.split("---").next() {
                        let trimmed = u_msg.trim();
                        user_snippet = if trimmed.chars().count() > 120 {
                            format!("{}...", trimmed.chars().take(120).collect::<String>())
                        } else {
                            trimmed.to_string()
                        };
                    }
                }

                let ai_part = content.split("## 🤖 Atena Response").nth(1)
                    .or_else(|| content.split("## 🤖 Resposta da Atena").nth(1));

                if let Some(part) = ai_part {
                    let trimmed = part.trim();
                    ai_snippet = if trimmed.chars().count() > 140 {
                        format!("{}...", trimmed.chars().take(140).collect::<String>())
                    } else {
                        trimmed.to_string()
                    };
                }

                if !user_snippet.is_empty() {
                    lines.push(format!("- Link `{}`:", file_name));
                    lines.push(format!("  User: \"{}\"", user_snippet.replace('\n', " ")));
                    if !ai_snippet.is_empty() {
                        lines.push(format!("  Atena: \"{}\"", ai_snippet.replace('\n', " ")));
                    }
                }
            }
        }

        if lines.len() <= 1 {
            None
        } else {
            Some(lines.join("\n"))
        }
    }

    /// Returns all saved episodes in the chain, ordered from newest to oldest
    pub fn get_all_episodes() -> Vec<EpisodeItem> {
        let dir = Self::episodios_dir();
        if !dir.exists() {
            return Vec::new();
        }

        let mut entries = match std::fs::read_dir(&dir) {
            Ok(rd) => rd
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| {
                    p.extension().and_then(|s| s.to_str()) == Some("md")
                        && p.file_name()
                            .and_then(|s| s.to_str())
                            .map(|s| s.starts_with("episodio_"))
                            .unwrap_or(false)
                })
                .collect::<Vec<PathBuf>>(),
            Err(_) => return Vec::new(),
        };

        entries.sort();

        let mut list = Vec::new();

        for (idx_counter, path) in entries.iter().enumerate() {
            if let Ok(raw_markdown) = std::fs::read_to_string(path) {
                let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("episode.md").to_string();

                let id = raw_markdown
                    .lines()
                    .find(|l| l.contains("- **Episode ID**:") || l.contains("- **ID do Episódio**:"))
                    .and_then(|l| l.split('`').nth(1))
                    .unwrap_or("")
                    .to_string();

                let date_time = raw_markdown
                    .lines()
                    .find(|l| l.contains("- **Date / Time**:") || l.contains("- **Data / Hora**:"))
                    .and_then(|l| {
                        l.split("**Date / Time**:").nth(1)
                            .or_else(|| l.split("**Data / Hora**:").nth(1))
                    })
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();

                let session_id = raw_markdown
                    .lines()
                    .find(|l| l.contains("- **Conversation ID**:") || l.contains("- **ID da Conversa**:"))
                    .and_then(|l| {
                        if let Some(start) = l.find('`') {
                            let rest = &l[start + 1..];
                            rest.split('`').next().map(|s| s.to_string())
                        } else {
                            l.split("**Conversation ID**:").nth(1)
                                .or_else(|| l.split("**ID da Conversa**:").nth(1))
                                .map(|s| s.trim().to_string())
                        }
                    })
                    .filter(|s| !s.is_empty() && s != "None");

                let session_title = raw_markdown
                    .lines()
                    .find(|l| l.contains("- **Conversation Title**:") || l.contains("- **Título da Conversa**:"))
                    .and_then(|l| {
                        l.split("**Conversation Title**:").nth(1)
                            .or_else(|| l.split("**Título da Conversa**:").nth(1))
                    })
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty() && s != "None");

                let prev_file = raw_markdown
                    .lines()
                    .find(|l| l.contains("- **Previous Link**:") || l.contains("- **Elo Anterior**:"))
                    .and_then(|l| {
                        if let Some(start) = l.find("./") {
                            let rest = &l[start + 2..];
                            rest.split(')').next().map(|s| s.to_string())
                        } else {
                            None
                        }
                    });

                let next_file = raw_markdown
                    .lines()
                    .find(|l| l.contains("- **Next Link**:") || l.contains("- **Elo Seguinte**:"))
                    .and_then(|l| {
                        if let Some(start) = l.find("./") {
                            let rest = &l[start + 2..];
                            rest.split(')').next().map(|s| s.to_string())
                        } else {
                            None
                        }
                    });

                let mut learned_memories = Vec::new();
                let mem_section = raw_markdown.split("- **Memories Learned in This Turn**:").nth(1)
                    .or_else(|| raw_markdown.split("- **Memórias Aprendidas Neste Turno**:").nth(1));
                if let Some(section) = mem_section {
                    if let Some(mem_content) = section.split("---").next() {
                        for line in mem_content.lines() {
                            let trimmed = line.trim();
                            if trimmed.starts_with("- ")
                                && !trimmed.contains("No new declarative memories")
                                && !trimmed.contains("Nenhuma nova memória") {
                                learned_memories.push(trimmed.trim_start_matches("- ").trim().to_string());
                            }
                        }
                    }
                }

                let user_message = raw_markdown
                    .split("## 👤 User Message")
                    .nth(1)
                    .or_else(|| raw_markdown.split("## 👤 Mensagem do Usuário").nth(1))
                    .and_then(|s| s.split("---").next())
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();

                let ai_response = raw_markdown
                    .split("## 🤖 Atena Response")
                    .nth(1)
                    .or_else(|| raw_markdown.split("## 🤖 Resposta da Atena").nth(1))
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();

                let index = raw_markdown
                    .lines()
                    .next()
                    .and_then(|l| l.split('#').nth(1))
                    .and_then(|s| s.trim().parse::<usize>().ok())
                    .unwrap_or(idx_counter + 1);

                list.push(EpisodeItem {
                    file_name,
                    index,
                    id,
                    date_time,
                    session_id,
                    session_title,
                    prev_file,
                    next_file,
                    learned_memories,
                    user_message,
                    ai_response,
                    raw_markdown,
                });
            }
        }

        list.reverse();
        list
    }

    /// Searches episodic history by keywords and/or session ID
    pub fn search_episodes(query: &str, session_id: Option<&str>, limit: usize) -> Vec<EpisodeItem> {
        let all = Self::get_all_episodes();
        let query_lower = query.to_lowercase().trim().to_string();
        let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

        let mut filtered: Vec<EpisodeItem> = all.into_iter().filter(|ep| {
            if let Some(sid) = session_id {
                if !sid.is_empty() && ep.session_id.as_deref() != Some(sid) {
                    return false;
                }
            }
            if query_terms.is_empty() {
                return true;
            }
            let search_target = format!(
                "{} {} {} {} {} {}",
                ep.id,
                ep.date_time,
                ep.session_title.as_deref().unwrap_or(""),
                ep.user_message,
                ep.ai_response,
                ep.learned_memories.join(" ")
            ).to_lowercase();

            query_terms.iter().any(|term| search_target.contains(term))
        }).collect();

        filtered.truncate(limit.max(1));
        filtered
    }

    /// Returns full details of an episode by its ID, filename, or index
    pub fn get_episode_detail(identifier: &str) -> Option<EpisodeItem> {
        let all = Self::get_all_episodes();
        let target = identifier.trim();
        all.into_iter().find(|ep| {
            ep.file_name == target
                || ep.id == target
                || ep.index.to_string() == target
                || ep.file_name.starts_with(target)
        })
    }

    /// Counts the number of episodes and memories generated in a specific session
    pub fn count_session_episodes_and_memories(session_id: &str) -> (usize, usize) {
        let all = Self::get_all_episodes();
        let mut ep_count = 0;
        let mut mem_count = 0;
        for ep in &all {
            if ep.session_id.as_deref() == Some(session_id) {
                ep_count += 1;
                mem_count += ep.learned_memories.len();
            }
        }
        (ep_count, mem_count)
    }

    /// Deletes episodes and/or memories associated with a specific chat session,
    /// repairing the chain links of remaining episodes when entries are removed.
    pub fn delete_session_episodes_and_memories(
        &mut self,
        session_id: &str,
        delete_episodes: bool,
        delete_memories: bool,
    ) -> Result<(usize, usize), String> {
        let all = Self::get_all_episodes();
        let dir = Self::episodios_dir();
        let mut deleted_eps = 0;
        let mut deleted_mems = 0;

        let mut memories_to_forget: Vec<String> = Vec::new();
        let mut files_to_delete: Vec<PathBuf> = Vec::new();

        for ep in &all {
            if ep.session_id.as_deref() == Some(session_id) {
                if delete_episodes {
                    files_to_delete.push(dir.join(&ep.file_name));
                    deleted_eps += 1;
                }
                if delete_memories {
                    for mem in &ep.learned_memories {
                        memories_to_forget.push(mem.clone());
                        deleted_mems += 1;
                    }
                }
            }
        }

        // 1. Forget associative memories in graph
        if delete_memories && !memories_to_forget.is_empty() {
            let mut dummy_learned = Vec::new();
            for mem_str in &memories_to_forget {
                let clean = mem_str
                    .replace("[MEMÓRIA SALVA & REFORÇADA]:", "")
                    .replace("[MEMÓRIA SALVA]:", "")
                    .replace("💚", "")
                    .replace("🧠", "");
                if let Some((subj, prop)) = clean.split_once("->") {
                    let s = subj.trim();
                    let p = prop.trim();
                    if !s.is_empty() && !p.is_empty() {
                        self.apply_forget_rule(s, p, &mut dummy_learned);
                    }
                }
            }
            let _ = self.auto_persist_default();
        }

        // 2. Delete episode files and repair chain links
        if delete_episodes && !files_to_delete.is_empty() {
            for f in &files_to_delete {
                let _ = std::fs::remove_file(f);
            }

            let mut remaining_entries = match std::fs::read_dir(&dir) {
                Ok(rd) => rd
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| {
                        p.extension().and_then(|s| s.to_str()) == Some("md")
                            && p.file_name()
                                .and_then(|s| s.to_str())
                                .map(|s| s.starts_with("episodio_"))
                                .unwrap_or(false)
                    })
                    .collect::<Vec<PathBuf>>(),
                Err(_) => Vec::new(),
            };

            remaining_entries.sort();

            let total_remaining = remaining_entries.len();
            let chain_info = EpisodicChainInfo {
                total_episodes: total_remaining,
                first_episode_file: remaining_entries.first().and_then(|p| p.file_name().and_then(|s| s.to_str()).map(|s| s.to_string())),
                last_episode_file: remaining_entries.last().and_then(|p| p.file_name().and_then(|s| s.to_str()).map(|s| s.to_string())),
                updated_at: Self::current_timestamp(),
            };

            for i in 0..remaining_entries.len() {
                let path = &remaining_entries[i];
                if let Ok(content) = std::fs::read_to_string(path) {
                    let prev_link = if i > 0 {
                        let prev_file = remaining_entries[i - 1].file_name().and_then(|s| s.to_str()).unwrap_or("");
                        format!("[Episode #{prev_idx}](./{prev_file})", prev_idx = i)
                    } else {
                        "Chain Start (First Episode)".to_string()
                    };

                    let next_link = if i + 1 < remaining_entries.len() {
                        let next_file = remaining_entries[i + 1].file_name().and_then(|s| s.to_str()).unwrap_or("");
                        format!("[Episode #{next_idx}](./{next_file})", next_idx = i + 2)
                    } else {
                        "(awaiting next link)".to_string()
                    };

                    let mut updated_lines = Vec::new();
                    for line in content.lines() {
                        if line.starts_with("- **Previous Link**:") || line.starts_with("- **Elo Anterior**:") {
                            updated_lines.push(format!("- **Previous Link**: {}", prev_link));
                        } else if line.starts_with("- **Next Link**:") || line.starts_with("- **Elo Seguinte**:") {
                            updated_lines.push(format!("- **Next Link**: {}", next_link));
                        } else {
                            updated_lines.push(line.to_string());
                        }
                    }
                    let _ = std::fs::write(path, updated_lines.join("\n"));
                }
            }

            let _ = Self::save_chain_info(&chain_info);
        }

        Ok((deleted_eps, deleted_mems))
    }

    // =========================================================================
    // Procedural Memory: Skills & Tasks (Folder-Based Architecture)
    // =========================================================================

    /// Returns the base procedural skills folder ~/.atena/skills/
    pub fn skills_root_dir() -> PathBuf {
        #[cfg(target_os = "windows")]
        let base = std::env::var("USERPROFILE")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));
        #[cfg(not(target_os = "windows"))]
        let base = std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));

        let dir = base.join(".atena").join("skills");
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    /// Legacy skills.json path in prefrontal cortex
    pub fn legacy_skills_file_path() -> PathBuf {
        let root = Self::brain_root_path();
        root.join("prefrontal").join("skills.json")
    }

    /// Generates a filesystem-friendly slug for a skill name
    pub fn skill_slug(name: &str) -> String {
        let slug: String = name
            .trim()
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
            .collect();
        let parts: Vec<&str> = slug.split('-').filter(|s| !s.is_empty()).collect();
        if parts.is_empty() {
            format!("skill-{}", Self::current_timestamp())
        } else {
            parts.join("-")
        }
    }

    /// Generates a human-readable SKILL.md representation for a procedural skill
    pub fn generate_skill_markdown(skill: &ProceduralSkill) -> String {
        let mut md = String::new();
        md.push_str(&format!("# {}\n\n", skill.name));
        if !skill.description.is_empty() {
            md.push_str(&format!("> {}\n\n", skill.description));
        }
        md.push_str(&format!("- **Version**: v{}\n", skill.version));
        if !skill.triggers.is_empty() {
            md.push_str(&format!("- **Triggers**: {}\n", skill.triggers.join(", ")));
        }
        md.push_str(&format!("- **Executions Count**: {}\n\n", skill.executions_count));

        md.push_str("## Execution Steps\n\n");
        for step in &skill.steps {
            md.push_str(&format!("{}. {}\n", step.order, step.instruction));
            if let Some(cmd) = &step.command {
                md.push_str(&format!("   - Command: `{}`\n", cmd));
            }
            if let Some(script) = &step.script_file {
                md.push_str(&format!("   - Script File: `{}`\n", script));
            }
            if let Some(cwd) = &step.cwd {
                md.push_str(&format!("   - Working Dir: `{}`\n", cwd));
            }
        }
        md.push('\n');

        if !skill.refinement_notes.is_empty() {
            md.push_str("## Refinement History\n\n");
            for note in &skill.refinement_notes {
                md.push_str(&format!("- {}\n", note));
            }
            md.push('\n');
        }

        md
    }

    /// Saves a single procedural skill to its dedicated directory in ~/.atena/skills/<slug>/
    pub fn save_single_skill(skill: &mut ProceduralSkill) -> Result<(), String> {
        let root = Self::skills_root_dir();
        let folder_path = if let Some(existing) = &skill.folder_path {
            PathBuf::from(existing)
        } else {
            let slug = Self::skill_slug(&skill.name);
            root.join(slug)
        };

        std::fs::create_dir_all(&folder_path).map_err(|e| e.to_string())?;
        let scripts_dir = folder_path.join("scripts");
        let _ = std::fs::create_dir_all(&scripts_dir);

        // Detect existing scripts in scripts/ directory
        let mut detected_scripts = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&scripts_dir) {
            for entry in entries.flatten() {
                if entry.path().is_file() {
                    if let Some(name) = entry.file_name().to_str() {
                        if !name.starts_with('.') {
                            detected_scripts.push(name.to_string());
                        }
                    }
                }
            }
        }
        detected_scripts.sort();
        skill.scripts = detected_scripts;
        skill.folder_path = Some(folder_path.to_string_lossy().to_string());

        // Write SKILL.md
        let md = Self::generate_skill_markdown(skill);
        let _ = std::fs::write(folder_path.join("SKILL.md"), md);

        // Write skill.json
        let json = serde_json::to_string_pretty(skill).map_err(|e| e.to_string())?;
        std::fs::write(folder_path.join("skill.json"), json).map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Loads all procedural skills from their dedicated folders in ~/.atena/skills/
    pub fn load_skills() -> Vec<ProceduralSkill> {
        // 1. Check and migrate legacy skills from ~/.atena/brain/prefrontal/skills.json
        let legacy_path = Self::legacy_skills_file_path();
        if legacy_path.exists() {
            if let Ok(data) = std::fs::read_to_string(&legacy_path) {
                if let Ok(legacy_skills) = serde_json::from_str::<Vec<ProceduralSkill>>(&data) {
                    log::info!("Migrating {} legacy skills into folder-based storage in ~/.atena/skills/...", legacy_skills.len());
                    for mut s in legacy_skills {
                        let _ = Self::save_single_skill(&mut s);
                    }
                    let _ = std::fs::remove_file(&legacy_path);
                }
            }
        }

        // 2. Scan ~/.atena/skills/ subdirectories
        let root = Self::skills_root_dir();
        let mut skills = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let json_path = path.join("skill.json");
                    if json_path.exists() {
                        if let Ok(content) = std::fs::read_to_string(&json_path) {
                            if let Ok(mut skill) = serde_json::from_str::<ProceduralSkill>(&content) {
                                skill.folder_path = Some(path.to_string_lossy().to_string());

                                let scripts_dir = path.join("scripts");
                                let mut detected_scripts = Vec::new();
                                if scripts_dir.is_dir() {
                                    if let Ok(s_entries) = std::fs::read_dir(&scripts_dir) {
                                        for se in s_entries.flatten() {
                                            if se.path().is_file() {
                                                if let Some(name) = se.file_name().to_str() {
                                                    if !name.starts_with('.') {
                                                        detected_scripts.push(name.to_string());
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                detected_scripts.sort();
                                skill.scripts = detected_scripts;

                                skills.push(skill);
                            }
                        }
                    }
                }
            }
        }

        skills.sort_by(|a, b| b.last_refined_at.cmp(&a.last_refined_at));
        skills
    }

    /// Saves the entire list of skills to folder-based directories
    pub fn save_skills(skills: &[ProceduralSkill]) -> Result<(), String> {
        for s in skills {
            let mut clone = s.clone();
            Self::save_single_skill(&mut clone)?;
        }
        Ok(())
    }

    /// Advanced skill creation or refinement with detailed steps and script support
    pub fn learn_or_refine_skill_advanced(
        id: Option<String>,
        name: &str,
        description: &str,
        triggers: Vec<String>,
        steps: Vec<SkillStep>,
        refinement_note: Option<String>,
        env_vars: Option<std::collections::HashMap<String, String>>,
        permission_mode: Option<String>,
    ) -> ProceduralSkill {
        let mut skills = Self::load_skills();
        let now = Self::current_timestamp();
        let clean_name = name.trim();

        // Check if updating an existing skill by ID or exact name
        let existing_idx = if let Some(target_id) = &id {
            skills.iter().position(|s| s.id == *target_id)
        } else {
            skills.iter().position(|s| s.name.eq_ignore_ascii_case(clean_name))
        };

        if let Some(idx) = existing_idx {
            let existing = &mut skills[idx];
            existing.version += 1;
            existing.name = clean_name.to_string();
            if !description.trim().is_empty() {
                existing.description = description.trim().to_string();
            }
            if !steps.is_empty() {
                existing.steps = steps;
            }
            for t in triggers {
                let t_clean = t.trim().to_string();
                if !t_clean.is_empty() && !existing.triggers.iter().any(|x| x.eq_ignore_ascii_case(&t_clean)) {
                    existing.triggers.push(t_clean);
                }
            }
            if env_vars.is_some() {
                existing.env_vars = env_vars;
            }
            if let Some(pm) = permission_mode {
                existing.permission_mode = pm;
            }
            existing.last_refined_at = now;
            if let Some(note) = refinement_note {
                let n_clean = note.trim().to_string();
                if !n_clean.is_empty() {
                    existing.refinement_notes.push(format!("v{}: {}", existing.version, n_clean));
                }
            }
            let mut updated = existing.clone();
            let _ = Self::save_single_skill(&mut updated);
            return updated;
        }

        // Creating brand new skill
        let new_id = format!("skill_{}", now);
        let mut notes = Vec::new();
        if let Some(note) = refinement_note {
            if !note.trim().is_empty() {
                notes.push(format!("v1: {}", note.trim()));
            }
        }

        let mut new_skill = ProceduralSkill {
            id: new_id,
            name: clean_name.to_string(),
            description: description.trim().to_string(),
            triggers: triggers.into_iter().map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect(),
            steps,
            version: 1,
            executions_count: 0,
            success_count: 0,
            last_refined_at: now,
            refinement_notes: notes,
            folder_path: None,
            scripts: Vec::new(),
            env_vars,
            permission_mode: permission_mode.unwrap_or_else(|| "ask".to_string()),
        };

        let _ = Self::save_single_skill(&mut new_skill);
        new_skill
    }

    /// Creates or refines a skill with plain string steps (backward compatible, parsing inline [cmd: ...] or [script: ...])
    pub fn learn_or_refine_skill(
        name: &str,
        description: &str,
        triggers: Vec<String>,
        raw_steps: Vec<String>,
        refinement_note: Option<String>,
    ) -> ProceduralSkill {
        let mut steps: Vec<SkillStep> = Vec::new();
        for (i, step_str) in raw_steps.iter().enumerate() {
            let s_clean = step_str.trim();
            if !s_clean.is_empty() {
                let mut instruction = s_clean.to_string();
                let mut cmd = None;
                let mut script = None;

                if let Some(c_start) = instruction.find("[cmd:") {
                    if let Some(c_end) = instruction[c_start..].find(']') {
                        cmd = Some(instruction[c_start + 5..c_start + c_end].trim().to_string());
                        instruction.replace_range(c_start..=c_start + c_end, "");
                    }
                } else if let Some(c_start) = instruction.find("[command:") {
                    if let Some(c_end) = instruction[c_start..].find(']') {
                        cmd = Some(instruction[c_start + 9..c_start + c_end].trim().to_string());
                        instruction.replace_range(c_start..=c_start + c_end, "");
                    }
                }

                if cmd.is_none() {
                    if let Some(start) = instruction.find('`') {
                        if let Some(end) = instruction[start + 1..].find('`') {
                            let extracted = instruction[start + 1..start + 1 + end].trim();
                            if !extracted.is_empty() {
                                cmd = Some(extracted.to_string());
                            }
                        }
                    }
                }

                if let Some(s_start) = instruction.find("[script:") {
                    if let Some(s_end) = instruction[s_start..].find(']') {
                        script = Some(instruction[s_start + 8..s_start + s_end].trim().to_string());
                        instruction.replace_range(s_start..=s_start + s_end, "");
                    }
                }

                steps.push(SkillStep {
                    order: (i + 1) as u32,
                    instruction: instruction.trim().to_string(),
                    tool_name: None,
                    command: cmd,
                    script_file: script,
                    cwd: None,
                    timeout_ms: None,
                });
            }
        }

        Self::learn_or_refine_skill_advanced(
            None,
            name,
            description,
            triggers,
            steps,
            refinement_note,
            None,
            None,
        )
    }

    /// Creates or updates a procedural skill complete with runnable script files and safe execution validation
    pub fn create_skill_with_scripts(
        name: &str,
        description: &str,
        triggers: Vec<String>,
        steps: Vec<SkillStep>,
        scripts: Vec<SkillScriptFilePayload>,
        env_vars: Option<HashMap<String, String>>,
    ) -> Result<ProceduralSkill, String> {
        let clean_name = name.trim();
        if clean_name.is_empty() {
            return Err("Skill name cannot be empty".to_string());
        }

        // 1. Validate safety of all commands in steps
        for step in &steps {
            if let Some(cmd) = &step.command {
                crate::services::skill_runner::SkillScriptRunner::validate_command_safety(cmd)?;
            }
        }

        // 2. Validate safety of script contents
        for script in &scripts {
            crate::services::skill_runner::SkillScriptRunner::validate_command_safety(&script.content)?;
        }

        // 3. Prepare skill directory in ~/.atena/skills/<slug>/
        let root = Self::skills_root_dir();
        let slug = Self::skill_slug(clean_name);
        let folder_path = root.join(&slug);
        std::fs::create_dir_all(&folder_path).map_err(|e| e.to_string())?;

        let scripts_dir = folder_path.join("scripts");
        std::fs::create_dir_all(&scripts_dir).map_err(|e| e.to_string())?;

        // 4. Write script files to scripts/
        let mut script_filenames = Vec::new();
        for script in scripts {
            let clean_filename = script.filename.trim()
                .replace('\\', "/")
                .split('/')
                .last()
                .unwrap_or("")
                .trim()
                .to_string();

            if clean_filename.is_empty() || clean_filename.starts_with('.') {
                continue;
            }

            let script_path = scripts_dir.join(&clean_filename);
            std::fs::write(&script_path, &script.content).map_err(|e| e.to_string())?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755));
            }

            script_filenames.push(clean_filename);
        }

        // 5. Versioning: check if existing skill
        let existing_skills = Self::load_skills();
        let target_id = format!("skill-{}", slug);
        let existing = existing_skills.iter().find(|s| s.name.to_lowercase() == clean_name.to_lowercase() || s.id == target_id);
        let version = if let Some(e) = existing {
            e.version + 1
        } else {
            1
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut skill = ProceduralSkill {
            id: target_id,
            name: clean_name.to_string(),
            description: description.trim().to_string(),
            version,
            triggers: triggers.into_iter().map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect(),
            steps,
            executions_count: existing.map(|e| e.executions_count).unwrap_or(0),
            success_count: existing.map(|e| e.success_count).unwrap_or(0),
            last_refined_at: now,
            refinement_notes: vec!["Skill created with automated scripts via conversation".to_string()],
            folder_path: Some(folder_path.to_string_lossy().to_string()),
            scripts: script_filenames,
            env_vars,
            permission_mode: existing.map(|e| e.permission_mode.clone()).unwrap_or_else(|| "ask".to_string()),
        };

        Self::save_single_skill(&mut skill)?;
        Ok(skill)
    }

    /// Updates, refines, or edits an existing procedural skill by ID or name
    pub fn update_skill_with_scripts(
        id: Option<String>,
        name: Option<String>,
        description: Option<String>,
        triggers: Option<Vec<String>>,
        steps: Option<Vec<SkillStep>>,
        scripts: Option<Vec<SkillScriptFilePayload>>,
        refinement_note: Option<String>,
        env_vars: Option<HashMap<String, String>>,
        permission_mode: Option<String>,
    ) -> Result<ProceduralSkill, String> {
        let skills = Self::load_skills();
        let target_id = id.as_deref().unwrap_or("").trim();
        let target_name = name.as_deref().unwrap_or("").trim();

        if target_id.is_empty() && target_name.is_empty() {
            return Err("Either skill ID or name must be provided to update a skill".to_string());
        }

        let existing_opt = skills.into_iter().find(|s| {
            (!target_id.is_empty() && (s.id.eq_ignore_ascii_case(target_id) || s.id == format!("skill-{}", target_id)))
                || (!target_name.is_empty() && (s.name.eq_ignore_ascii_case(target_name) || s.id == format!("skill-{}", Self::skill_slug(target_name))))
        });

        let mut skill = match existing_opt {
            Some(s) => s,
            None => {
                return Err(format!(
                    "Procedural skill '{}' not found for editing",
                    if !target_id.is_empty() { target_id } else { target_name }
                ));
            }
        };

        // 1. Update name if provided
        if !target_name.is_empty() {
            skill.name = target_name.to_string();
        }

        // 2. Update description if provided
        if let Some(desc) = description {
            if !desc.trim().is_empty() {
                skill.description = desc.trim().to_string();
            }
        }

        // 3. Update triggers if provided
        if let Some(new_triggers) = triggers {
            let mut merged_triggers = skill.triggers.clone();
            for t in new_triggers {
                let clean = t.trim().to_string();
                if !clean.is_empty() && !merged_triggers.iter().any(|x| x.eq_ignore_ascii_case(&clean)) {
                    merged_triggers.push(clean);
                }
            }
            if !merged_triggers.is_empty() {
                skill.triggers = merged_triggers;
            }
        }

        // 4. Update steps if provided
        if let Some(new_steps) = steps {
            if !new_steps.is_empty() {
                for step in &new_steps {
                    if let Some(cmd) = &step.command {
                        crate::services::skill_runner::SkillScriptRunner::validate_command_safety(cmd)?;
                    }
                }
                skill.steps = new_steps;
            }
        }

        // 5. Update scripts if provided
        if let Some(new_scripts) = scripts {
            if !new_scripts.is_empty() {
                for script in &new_scripts {
                    crate::services::skill_runner::SkillScriptRunner::validate_command_safety(&script.content)?;
                }

                let folder_path = match &skill.folder_path {
                    Some(fp) => PathBuf::from(fp),
                    None => {
                        let root = Self::skills_root_dir();
                        let slug = Self::skill_slug(&skill.name);
                        let fp = root.join(&slug);
                        let _ = std::fs::create_dir_all(&fp);
                        skill.folder_path = Some(fp.to_string_lossy().to_string());
                        fp
                    }
                };

                let scripts_dir = folder_path.join("scripts");
                std::fs::create_dir_all(&scripts_dir).map_err(|e| e.to_string())?;

                let mut current_scripts = skill.scripts.clone();
                for script in new_scripts {
                    let clean_filename = script.filename.trim()
                        .replace('\\', "/")
                        .split('/')
                        .last()
                        .unwrap_or("")
                        .trim()
                        .to_string();

                    if clean_filename.is_empty() || clean_filename.starts_with('.') {
                        continue;
                    }

                    let script_path = scripts_dir.join(&clean_filename);
                    std::fs::write(&script_path, &script.content).map_err(|e| e.to_string())?;

                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let _ = std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755));
                    }

                    if !current_scripts.iter().any(|s| s.eq_ignore_ascii_case(&clean_filename)) {
                        current_scripts.push(clean_filename);
                    }
                }
                skill.scripts = current_scripts;
            }
        }

        // 6. Update env vars & permission mode
        if env_vars.is_some() {
            skill.env_vars = env_vars;
        }
        if let Some(pm) = permission_mode {
            skill.permission_mode = pm;
        }

        // 7. Bump version & record refinement
        skill.version += 1;
        skill.last_refined_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let note_text = refinement_note
            .map(|n| n.trim().to_string())
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| "Skill updated and refined via conversation".to_string());
        skill.refinement_notes.push(format!("v{}: {}", skill.version, note_text));

        Self::save_single_skill(&mut skill)?;
        Ok(skill)
    }

    /// Deletes a procedural skill and its folder by ID
    pub fn delete_skill(id: &str) -> Result<(), String> {
        let skills = Self::load_skills();
        if let Some(target) = skills.iter().find(|s| s.id == id) {
            if let Some(path_str) = &target.folder_path {
                let path = PathBuf::from(path_str);
                if path.exists() && path.is_dir() {
                    let _ = std::fs::remove_dir_all(&path);
                }
            }
            Ok(())
        } else {
            Err(format!("Skill with id '{}' not found", id))
        }
    }

    /// Searches for skills matching a user prompt
    pub fn find_matching_skills(query: &str) -> Vec<ProceduralSkill> {
        let skills = Self::load_skills();
        let q_lower = query.to_lowercase();
        let words: Vec<&str> = q_lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() >= 3)
            .collect();

        skills
            .into_iter()
            .filter(|s| {
                let name_match = q_lower.contains(&s.name.to_lowercase());
                let trigger_match = s.triggers.iter().any(|t| {
                    let t_low = t.to_lowercase();
                    q_lower.contains(&t_low) || words.iter().any(|w| t_low.contains(*w))
                });
                name_match || trigger_match
            })
            .collect()
    }

    /// Builds the skills prompt context block with execution directives and safety rules
    pub fn build_skills_prompt_context(query: &str) -> Option<String> {
        let matches = Self::find_matching_skills(query);
        if matches.is_empty() {
            return None;
        }

        let mut out = String::from("\n[LEARNED PROCEDURAL SKILLS & WORKFLOWS]:\n");
        out.push_str("You have learned the following exact steps to perform these tasks. Follow this execution workflow strictly.\n");
        out.push_str("SAFETY DIRECTIVE: NEVER generate or run destructive commands (such as recursive file deletion `rm -rf`, disk formatting, or raw partitions). Always employ safe, non-destructive commands.\n\n");

        let mut has_executable = false;

        for skill in matches {
            out.push_str(&format!("🛠️ SKILL: {} (Version v{})\n", skill.name, skill.version));
            if !skill.description.is_empty() {
                out.push_str(&format!("   Description: {}\n", skill.description));
            }
            if let Some(folder) = &skill.folder_path {
                out.push_str(&format!("   Skill Directory: {}\n", folder));
            }
            if !skill.scripts.is_empty() {
                out.push_str(&format!("   Available Scripts: {}\n", skill.scripts.join(", ")));
            }
            out.push_str("   Execution Steps:\n");
            for step in &skill.steps {
                let mut line = format!("     {}. {}", step.order, step.instruction);
                let eff_cmd = step.effective_command();
                if let Some(cmd) = &eff_cmd {
                    line.push_str(&format!(" [Command: `{}`]", cmd));
                    has_executable = true;
                }
                if let Some(script) = &step.script_file {
                    line.push_str(&format!(" [Script: `{}`]", script));
                    has_executable = true;
                }
                out.push_str(&line);
                out.push('\n');
            }
            if !skill.refinement_notes.is_empty() {
                out.push_str(&format!("   Latest Refinement: {}\n", skill.refinement_notes.last().unwrap()));
            }
            out.push('\n');
        }

        if has_executable {
            out.push_str("[HOW TO EXECUTE SKILL COMMANDS & SCRIPTS]:\n");
            out.push_str("To execute a command, emit a tool call in your response:\n");
            out.push_str("<tool_call>\n{\"name\": \"run_command\", \"arguments\": {\"command\": \"<exact command>\"}}\n</tool_call>\n\n");
            out.push_str("Or to execute a script located in the skill's scripts directory:\n");
            out.push_str("<tool_call>\n{\"name\": \"run_skill_script\", \"arguments\": {\"slug\": \"<skill-slug>\", \"script_file\": \"<filename>\", \"args\": []}}\n</tool_call>\n\n");
            out.push_str("CRITICAL INSTRUCTIONS FOR COMMAND TOOL CALLS:\n");
            out.push_str("1. NEVER use the title or name of any skill as the tool \"name\". Doing so will cause an execution error!\n");
            out.push_str("2. The \"name\" field MUST ALWAYS BE strictly \"run_command\" (for terminal commands) or \"run_skill_script\" (for scripts). \"run_skill_command\" is also accepted for backward compatibility.\n");
            out.push_str("3. Use \"run_command\" whenever you need to execute CLI commands for these skills, or safe system inspection/diagnostics requested by the user.\n");
            out.push_str("4. Once command output is returned, formulate your final response to the user. DO NOT emit `create_procedural_skill` to re-save or duplicate an already learned skill! If the user asks to modify, refine, or update an existing skill, use `update_procedural_skill`.\n");
            out.push_str("Atena Studio will intercept this tool call, check authorizations (or run automatically if configured with auto permission), run the command safely, and return the output (stdout/stderr) back to you so you can formulate your final response to the user.\n");
        }

        Some(out)
    }

    // =========================================================================
    // Aprendizado Semântico / Heurístico a partir de Diálogos
    // =========================================================================

    /// Analyzes text for facts and entities, adding to global graph and persisting
    pub fn learn_from_text(&mut self, text: &str) -> Vec<String> {
        self.learn_from_text_with_session(text, None)
    }

    /// Analyzes text for facts and entities with optional session scope
    pub fn learn_from_text_with_session(&mut self, text: &str, session_id: Option<&str>) -> Vec<String> {
        let mut learned = Vec::new();
        let clean = text.trim();
        if clean.is_empty() {
            return learned;
        }

        let start_node_id = self.next_id;

        // 0. Process XML memory tags (<memoria ... /> or <memory ... />) first if present
        if clean.contains("<memoria") || clean.contains("<memory") || clean.contains("<memorizar") {
            let (_, mut tag_learned) = self.extract_and_apply_memory_tags(clean);
            learned.append(&mut tag_learned);
        }

        // 1. Identificar ou recuperar o usuário principal (âncora pessoal)
        let mut user_entity: Option<String> = None;
        let lower_full = clean.to_lowercase();

        for pattern in &[
            "me chamo ", "meu nome é ", "meu nome e ", "meu nome eh ", "sou o ", "sou a ", "me chamam de ",
            "my name is ", "i am ", "i'm ", "call me ", "they call me "
        ] {
            if let Some(pos) = lower_full.find(pattern) {
                let start = pos + pattern.len();
                let rest = &clean[start..];
                let name = rest
                    .split(|c: char| c == ',' || c == '.' || c == '\n' || c == ';' || c == '!' || c == '?')
                    .next()
                    .unwrap_or("")
                    .trim();
                let first_word = name.split_whitespace().next().unwrap_or("").trim();
                if !first_word.is_empty() && first_word.len() > 1 {
                    let cap_name = first_word.to_string();
                    user_entity = Some(cap_name.clone());
                    let node_id = self.get_or_create_node(NodeType::Object, &cap_name);
                    learned.push(format!("Entidade identificada: {}", cap_name));

                    let attr_id = self.get_or_create_node(NodeType::Attribute, &format!("Name: {}", cap_name));
                    let _ = self.add_edge(node_id, attr_id, RelationType::HasProperty);
                }
            }
        }

        // If no explicit name found in current text, recover existing primary entity node
        let anchor_user = user_entity.or_else(|| {
            self.nodes
                .values()
                .find(|n| n.type_flag == NodeType::Object && n.label.chars().next().map_or(false, |c| c.is_uppercase()))
                .map(|n| n.label.clone())
        });

        // 2. Split text into clauses/sentences for contextual processing
        let clauses: Vec<&str> = clean
            .split(|c: char| c == '.' || c == '\n' || c == ';' || c == '!' || c == '?' || c == '|')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        // Rastrear última entidade mencionada (para pronomes "ela", "ele")
        let mut last_female_entity: Option<String> = None;
        let mut last_male_entity: Option<String> = None;

        // Pre-scan: Detect proper names and interpersonal relationships in clauses
        for clause in &clauses {
            let cl_lower = clause.to_lowercase();

            // Padrão: "[NOME] e/é/eh minha esposa/mulher..." ou "minha esposa e/é/eh [NOME]"
            for rel_word in &["esposa", "mulher", "namorada", "noiva", "mãe", "mae", "irmã", "irma", "amiga", "sócia", "socia"] {
                if cl_lower.contains(rel_word) {
                    let parts: Vec<&str> = clause.split(',').collect();
                    for part in parts {
                        let p_trim = part.trim();
                        let p_lower = p_trim.to_lowercase();
                        if p_lower.contains(rel_word) {
                            for conn in &[" e minha", " é minha", " eh minha", " e a minha", " é a minha", " eh a minha"] {
                                if let Some(pos) = p_lower.find(conn) {
                                    let name = p_trim[..pos].trim();
                                    if !name.is_empty() && name.chars().next().map_or(false, |c| c.is_uppercase()) {
                                        let node_id = self.get_or_create_node(NodeType::Object, name);
                                        last_female_entity = Some(name.to_string());
                                        learned.push(format!("Entidade identificada: {}", name));

                                        let rel_name = format!("{}: {}", rel_word.to_uppercase(), name);
                                        let rel_attr = self.get_or_create_node(NodeType::Attribute, &rel_name);

                                        if let Some(ref u) = anchor_user {
                                            let u_id = self.get_or_create_node(NodeType::Object, u);
                                            let _ = self.add_edge(u_id, node_id, RelationType::HasProperty);
                                            let _ = self.add_edge(u_id, rel_attr, RelationType::HasProperty);
                                            let _ = self.add_edge(node_id, rel_attr, RelationType::HasProperty);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Padrão: "[NOME] e/é/eh meu marido/esposo/namorado/noivo/irmão/pai/filho/amigo/sócio"
            for rel_word in &["marido", "esposo", "namorado", "noivo", "pai", "irmão", "irmao", "filho", "amigo", "sócio", "socio"] {
                if cl_lower.contains(rel_word) {
                    for conn in &[" e meu", " é meu", " eh meu", " e o meu", " é o meu", " eh o meu"] {
                        if let Some(pos) = cl_lower.find(conn) {
                            let name = clause[..pos].trim();
                            if !name.is_empty() && name.chars().next().map_or(false, |c| c.is_uppercase()) {
                                let node_id = self.get_or_create_node(NodeType::Object, name);
                                last_male_entity = Some(name.to_string());
                                learned.push(format!("Entidade identificada: {}", name));

                                let rel_name = format!("{}: {}", rel_word.to_uppercase(), name);
                                let rel_attr = self.get_or_create_node(NodeType::Attribute, &rel_name);

                                if let Some(ref u) = anchor_user {
                                    let u_id = self.get_or_create_node(NodeType::Object, u);
                                    let _ = self.add_edge(u_id, node_id, RelationType::HasProperty);
                                    let _ = self.add_edge(u_id, rel_attr, RelationType::HasProperty);
                                }
                            }
                        }
                    }
                }
            }

            // Padrão: "temos uma filha/filho [chamada/chamado/camada] [NOME]" ou "minha filha [NOME]"
            for child_word in &["filha", "filho"] {
                if cl_lower.contains(child_word) {
                    for call_word in &["chamada ", "chamado ", "camada ", "camado ", "de nome ", "que se chama "] {
                        if let Some(pos) = cl_lower.find(call_word) {
                            let start = pos + call_word.len();
                            let rest = &clause[start..];
                            let name = rest
                                .split(|c: char| c == ',' || c == '.' || c == '\n' || c == ';')
                                .next()
                                .unwrap_or("")
                                .trim();
                            if !name.is_empty() {
                                let node_id = self.get_or_create_node(NodeType::Object, name);
                                if *child_word == "filha" {
                                    last_female_entity = Some(name.to_string());
                                } else {
                                    last_male_entity = Some(name.to_string());
                                }
                                learned.push(format!("Filho(a) identificado(a): {}", name));

                                let rel_name = format!("{}: {}", child_word.to_uppercase(), name);
                                let rel_attr = self.get_or_create_node(NodeType::Attribute, &rel_name);

                                // Ligar ao usuário principal
                                if let Some(ref u) = anchor_user {
                                    let u_id = self.get_or_create_node(NodeType::Object, u);
                                    let _ = self.add_edge(u_id, node_id, RelationType::HasProperty);
                                    let _ = self.add_edge(u_id, rel_attr, RelationType::HasProperty);
                                }

                                // Ligar à esposa/marido se existir
                                if let Some(ref f) = last_female_entity {
                                    if f != name {
                                        let f_id = self.get_or_create_node(NodeType::Object, f);
                                        let _ = self.add_edge(f_id, node_id, RelationType::HasProperty);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Process clauses and attributes associating with correct subject
        let sub_clauses: Vec<&str> = clean
            .split(|c: char| c == ',' || c == '.' || c == '\n' || c == ';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        for sub in &sub_clauses {
            let s_lower = sub.to_lowercase();

            // Determine clause subject:
            let current_subject = if s_lower.starts_with("ela ") || s_lower.contains("dela") || s_lower.contains("esposa") {
                last_female_entity.clone().or_else(|| anchor_user.clone())
            } else if s_lower.starts_with("ele ") || s_lower.contains("dele") || s_lower.contains("marido") {
                last_male_entity.clone().or_else(|| anchor_user.clone())
            } else {
                // Verify if clause begins with a known entity name
                let mut matched_subj = None;
                for node in self.nodes.values() {
                    if node.type_flag == NodeType::Object {
                        let node_label_lower = node.label.to_lowercase();
                        if s_lower.starts_with(&node_label_lower) {
                            matched_subj = Some(node.label.clone());
                            break;
                        }
                    }
                }
                matched_subj.or_else(|| anchor_user.clone())
            };

            if let Some(ref subj_name) = current_subject {
                let subj_id = self.get_or_create_node(NodeType::Object, subj_name);

                // Idade: "tenho 23 anos" / "tem 23 anos"
                if let Some(pos) = s_lower.find(" anos") {
                    let before = &s_lower[..pos];
                    if let Some(num_str) = before.split(|c: char| !c.is_numeric()).filter(|s| !s.is_empty()).last() {
                        let age_str = format!("{} anos", num_str);
                        let age_id = self.get_or_create_node(NodeType::Attribute, &age_str);
                        let _ = self.add_edge(subj_id, age_id, RelationType::HasProperty);
                        learned.push(format!("{} tem a propriedade: {}", subj_name, age_str));
                    }
                }

                // Cabelos: "cabelos castanhos", "cabelo preto", "cabelos pretos"
                if let Some(pos) = s_lower.find("cabelo") {
                    let start = if s_lower[pos..].starts_with("cabelos ") { pos + 8 } else { pos + 7 };
                    let rest = &sub[start..].trim();
                    let color = rest.split_whitespace().next().unwrap_or("").trim();
                    if !color.is_empty() {
                        let attr = format!("Cabelos {}", color);
                        let attr_id = self.get_or_create_node(NodeType::Attribute, &attr);
                        let _ = self.add_edge(subj_id, attr_id, RelationType::HasProperty);
                        learned.push(format!("{} -> {}", subj_name, attr));
                    }
                }

                // Olhos: "olhos marrons", "olhos castanhos", "olhos marrom"
                if let Some(pos) = s_lower.find("olho") {
                    if s_lower.contains("da cor do meu") {
                        let attr = "Olhos castanhos / marrons".to_string();
                        let attr_id = self.get_or_create_node(NodeType::Attribute, &attr);
                        let _ = self.add_edge(subj_id, attr_id, RelationType::HasProperty);
                        learned.push(format!("{} -> {}", subj_name, attr));
                    } else {
                        let start = if s_lower[pos..].starts_with("olhos ") { pos + 6 } else { pos + 5 };
                        let rest = &sub[start..].trim();
                        let color = rest.split_whitespace().next().unwrap_or("").trim();
                        if !color.is_empty() && color != "da" && color != "de" {
                            let attr = format!("Olhos {}", color);
                            let attr_id = self.get_or_create_node(NodeType::Attribute, &attr);
                            let _ = self.add_edge(subj_id, attr_id, RelationType::HasProperty);
                            learned.push(format!("{} -> {}", subj_name, attr));
                        }
                    }
                }

                // Pele: "pele parda", "cor de pele parda"
                if let Some(pos) = s_lower.find("pele ") {
                    let rest = &sub[pos + 5..].trim();
                    let tone = rest.split_whitespace().next().unwrap_or("").trim();
                    if !tone.is_empty() {
                        let attr = format!("Pele {}", tone);
                        let attr_id = self.get_or_create_node(NodeType::Attribute, &attr);
                        let _ = self.add_edge(subj_id, attr_id, RelationType::HasProperty);
                        learned.push(format!("{} -> {}", subj_name, attr));
                    }
                }

                // Aparelho nos dentes
                if s_lower.contains("aparelho") {
                    let attr = "Usa aparelho nos dentes".to_string();
                    let attr_id = self.get_or_create_node(NodeType::Attribute, &attr);
                    let _ = self.add_edge(subj_id, attr_id, RelationType::HasProperty);
                    learned.push(format!("{} -> {}", subj_name, attr));
                }

                // Sorriso lindo
                if s_lower.contains("sorriso lindo") || s_lower.contains("lindo sorriso") {
                    let attr = "Sorriso lindo".to_string();
                    let attr_id = self.get_or_create_node(NodeType::Attribute, &attr);
                    let _ = self.add_edge(subj_id, attr_id, RelationType::HasProperty);
                    learned.push(format!("{} -> {}", subj_name, attr));
                }
            }
        }

        // 4. Storage and Location
        for prep in &["guardei ", "coloquei ", "deixei ", "guardado na ", "guardado no ", "guardada na ", "guardada no "] {
            if let Some(pos) = lower_full.find(prep) {
                let rest = &clean[pos + prep.len()..];
                for loc_prep in &[" na ", " no ", " em ", " dentro do ", " dentro da "] {
                    if let Some(loc_pos) = rest.to_lowercase().find(loc_prep) {
                        let obj_raw = rest[..loc_pos].trim();
                        let loc_raw = rest[loc_pos + loc_prep.len()..]
                            .split(|c: char| c == '.' || c == '\n' || c == ',' || c == ';')
                            .next()
                            .unwrap_or("")
                            .trim();
                        if !obj_raw.is_empty() && !loc_raw.is_empty() {
                            let obj_id = self.get_or_create_node(NodeType::Object, obj_raw);
                            let loc_id = self.get_or_create_node(NodeType::Container, loc_raw);
                            let _ = self.add_edge(obj_id, loc_id, RelationType::LocatedIn);
                            learned.push(format!("{} localizado em {}", obj_raw, loc_raw));
                        }
                    }
                }
            }
        }

        // 5. General Entities, Definitions, Background, Profession, Residence, and Predicates
        // Trata: "Maria e formada", "Maria é desenvolvedora", "Maria mora em SP", "Carlos Drummond é poeta"
        for clause in &clauses {
            let cl_trim = clause.trim();
            let cl_lower = cl_trim.to_lowercase();

            // List of common bilingual connectors and verbs (PT/EN)
            let connectors = [
                " é ", " e ", " eh ", " foi ", " era ", " seria ", " está ", " esta ", " ta ", " está em ", " esta em ",
                " is ", " was ", " will be ", " lives in ", " works as ", " works at ", " graduated in ", " studies ", " likes ", " loves ", " prefers ", " has ",
                " mora em ", " mora no ", " mora na ", " vive em ", " reside em ",
                " trabalha como ", " trabalha com ", " trabalha na ", " trabalha no ",
                " formado em ", " formada em ", " se formou em ", " graduado em ", " graduada em ",
                " estuda ", " cursa ", " gosta de ", " adora ", " prefere ", " tem ", " possui "
            ];

            let mut matched_connection = None;
            for conn in &connectors {
                if let Some(pos) = cl_lower.find(conn) {
                    matched_connection = Some((*conn, pos));
                    break;
                }
            }

            if let Some((conn, pos)) = matched_connection {
                let raw_subj = cl_trim[..pos].trim();
                let raw_pred = cl_trim[pos + conn.len()..].trim();

                // If subject is empty or personal pronoun ("I", "he", "she", "eu", "ele", "ela"), resolve subject
                let final_subj = if raw_subj.is_empty() || raw_subj.eq_ignore_ascii_case("eu") || raw_subj.eq_ignore_ascii_case("i") {
                    anchor_user.clone().unwrap_or_else(|| "User".to_string())
                } else if raw_subj.eq_ignore_ascii_case("ela") || raw_subj.eq_ignore_ascii_case("she") {
                    last_female_entity.clone().or_else(|| anchor_user.clone()).unwrap_or_else(|| "User".to_string())
                } else if raw_subj.eq_ignore_ascii_case("ele") || raw_subj.eq_ignore_ascii_case("he") {
                    last_male_entity.clone().or_else(|| anchor_user.clone()).unwrap_or_else(|| "User".to_string())
                } else {
                    raw_subj.to_string()
                };

                let clean_pred = raw_pred
                    .split(|c: char| c == ';' || c == '(' || c == ')')
                    .next()
                    .unwrap_or(raw_pred)
                    .trim();

                if !final_subj.is_empty() && !clean_pred.is_empty() && final_subj.len() <= 50 && clean_pred.len() <= 100 {
                    let subj_id = self.get_or_create_node(NodeType::Object, &final_subj);

                    // Format property according to connector
                    let (attr_label, rel_type, node_type) = if conn.contains("mora") || conn.contains("vive") || conn.contains("reside") || conn.contains("lives in") || (conn.contains("em ") && (conn.contains("está") || conn.contains("esta"))) {
                        (clean_pred.to_string(), RelationType::LocatedIn, NodeType::Container)
                    } else if conn.contains("formado em") || conn.contains("formou em") || conn.contains("graduado em") || conn.contains("graduated in") {
                        (format!("Graduated in {}", clean_pred), RelationType::HasProperty, NodeType::Attribute)
                    } else if conn.contains("trabalha") || conn.contains("works") {
                        (format!("Works: {}", clean_pred), RelationType::HasProperty, NodeType::Attribute)
                    } else if conn.contains("gosta") || conn.contains("adora") || conn.contains("prefere") || conn.contains("likes") || conn.contains("loves") || conn.contains("prefers") {
                        (format!("Likes {}", clean_pred), RelationType::HasProperty, NodeType::Attribute)
                    } else {
                        // " is ", " was ", " é ", " e ", etc.
                        let cap_pred = if let Some(first_char) = clean_pred.chars().next() {
                            format!("{}{}", first_char.to_uppercase(), &clean_pred[first_char.len_utf8()..])
                        } else {
                            clean_pred.to_string()
                        };
                        (cap_pred, RelationType::HasProperty, NodeType::Attribute)
                    };

                    let target_id = self.get_or_create_node(node_type, &attr_label);
                    let _ = self.add_edge(subj_id, target_id, rel_type);
                    learned.push(format!("{} -> {}", final_subj, attr_label));
                }
            }

            // Título / Alcunha: "conhecido pelo título de Rei do Baião"
            if let Some(pos) = cl_lower.find("conhecido como ").or_else(|| cl_lower.find("título de ")) {
                let start = if cl_lower[pos..].starts_with("conhecido como ") { pos + 15 } else { pos + 10 };
                if start < clause.len() {
                    let rest = &clause[start..];
                    let title = rest
                        .trim_start_matches(|c: char| c == '"' || c == '“' || c == '\'')
                        .split(|c: char| c == '"' || c == '”' || c == '\'' || c == '.' || c == ',' || c == '\n')
                        .next()
                        .unwrap_or("")
                        .trim();
                    if !title.is_empty() && title.len() <= 40 {
                        if let Some(ref subj) = anchor_user {
                            let subj_id = self.get_or_create_node(NodeType::Object, subj);
                            let attr_id = self.get_or_create_node(NodeType::Attribute, &format!("Título: {}", title));
                            let _ = self.add_edge(subj_id, attr_id, RelationType::HasProperty);
                            learned.push(format!("{} -> Título: {}", subj, title));
                        }
                    }
                }
            }
        }

        // 6. Fallback final para fatos diretos simples sem verbo reconhecido (ex: "Maria: Engenheira", "Moro no Brasil", etc.)
        if learned.is_empty() {
            if let Some(pos) = clean.find(':').or_else(|| clean.find(" - ")) {
                let sep_len = if clean[pos..].starts_with(':') { 1 } else { 3 };
                let subj = clean[..pos].trim();
                let val = clean[pos + sep_len..].trim();
                if !subj.is_empty() && !val.is_empty() && subj.len() <= 40 && val.len() <= 80 {
                    let subj_id = self.get_or_create_node(NodeType::Object, subj);
                    let attr_id = self.get_or_create_node(NodeType::Attribute, val);
                    let _ = self.add_edge(subj_id, attr_id, RelationType::HasProperty);
                    learned.push(format!("{} -> {}", subj, val));
                }
            } else if let Some(ref u) = anchor_user {
                // If user input is a short subject-less clause (e.g. "Degree in Computer Science")
                if clean.len() <= 60 && !clean.contains('\n') {
                    let u_id = self.get_or_create_node(NodeType::Object, u);
                    let attr_id = self.get_or_create_node(NodeType::Attribute, clean);
                    let _ = self.add_edge(u_id, attr_id, RelationType::HasProperty);
                    learned.push(format!("{} -> {}", u, clean));
                }
            }
        }

        // 6.1 If a session is provided, assign session_id to all new nodes created in this turn
        if let Some(sid) = session_id {
            let s_trimmed = sid.trim();
            if !s_trimmed.is_empty() {
                for id in start_node_id..self.next_id {
                    if let Some(node) = self.nodes.get_mut(&id) {
                        if node.session_id.is_none() {
                            node.session_id = Some(s_trimmed.to_string());
                        }
                    }
                }
            }
        }

        // 7. Se houve aprendizado, persiste automaticamente em disco (.atena)
        if !learned.is_empty() {
            let _ = self.auto_persist_default();
        }

        learned
    }

    // =========================================================================
    // Estatísticas
    // =========================================================================

    /// Retorna estatísticas do motor para debug/monitoramento
    pub fn stats(&self) -> EngineStats {
        EngineStats {
            total_nodes: self.nodes.len(),
            total_edges: self.edge_count(),
            cache_size: self.hot_cache.len(),
            cache_hits: self.hot_cache.hits,
            cache_misses: self.hot_cache.misses,
            cache_hit_rate: if self.hot_cache.hits + self.hot_cache.misses > 0 {
                self.hot_cache.hits as f64
                    / (self.hot_cache.hits + self.hot_cache.misses) as f64
                    * 100.0
            } else {
                0.0
            },
        }
    }

    // =========================================================================
    // Utilities
    // =========================================================================

    /// Returns the current Unix timestamp in seconds
    pub fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// Statistics for the memory graph engine
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EngineStats {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub cache_size: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_hit_rate: f64,
}

/// Query result returned to frontend
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryQueryResult {
    pub llm_context: String,
    pub paths: Vec<AssociationPath>,
    pub stats: EngineStats,
}

/// Complete graph data for real-time visualization in the interface
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FullGraphData {
    pub nodes: Vec<MemoryNode>,
    pub edges: Vec<MemoryEdge>,
    pub stats: EngineStats,
}

/// Metadata state of the Markdown episode chain
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EpisodicChainInfo {
    pub total_episodes: usize,
    pub first_episode_file: Option<String>,
    pub last_episode_file: Option<String>,
    pub updated_at: u64,
}

impl Default for EpisodicChainInfo {
    fn default() -> Self {
        Self {
            total_episodes: 0,
            first_episode_file: None,
            last_episode_file: None,
            updated_at: 0,
        }
    }
}

/// Structured item of an episode in the chain for UI display
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EpisodeItem {
    pub file_name: String,
    pub index: usize,
    pub id: String,
    pub date_time: String,
    pub session_id: Option<String>,
    pub session_title: Option<String>,
    pub prev_file: Option<String>,
    pub next_file: Option<String>,
    pub learned_memories: Vec<String>,
    pub user_message: String,
    pub ai_response: String,
    pub raw_markdown: String,
}

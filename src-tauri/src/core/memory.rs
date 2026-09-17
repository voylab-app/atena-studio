// =============================================================================
// core/memory.rs — Tipos e Estruturas do Sistema de Memória Associativa
// =============================================================================
//
// Define o layout binário compacto para nós, arestas e header do arquivo .atena.
// Todos os inteiros são serializados em little-endian.
//
// Byte Layout:
//   FileHeader: 15 bytes fixos (magic[4] + version[2] + compression[1] + size[8])
//   Node:       8 + label_len bytes (id[4] + type[1] + valence[1] + label_len[2] + label[N])
//   Edge:       25 bytes fixos (src[4] + tgt[4] + rel[1] + weight[4] + count[4] + ts[8])

use serde::{Deserialize, Serialize};

// =============================================================================
// Magic Bytes: "ATEN" (0x41 0x54 0x45 0x4E)
// =============================================================================
pub const MAGIC_BYTES: [u8; 4] = [0x41, 0x54, 0x45, 0x4E];
pub const CURRENT_VERSION: u16 = 3;

// =============================================================================
// Enums with explicit binary representation
// =============================================================================

/// Compression type for .atena file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum CompressionType {
    Uncompressed = 0x01,
    Zstd = 0x02,
    Lz4 = 0x03,
}

impl CompressionType {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Self::Uncompressed),
            0x02 => Some(Self::Zstd),
            0x03 => Some(Self::Lz4),
            _ => None,
        }
    }

    pub fn as_byte(self) -> u8 {
        self as u8
    }
}

/// Functional lobe of Atena's brain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum BrainLobe {
    /// Hippocampus: Waking buffer, event journal, and sleep/pruning queue
    Hipocampo = 0x05,
    /// Temporal Lobe: Names, identities, semantic facts, and episodes
    Temporal = 0x10,
    /// Parietal Lobe: Space, physical objects, containers, and locations
    Parietal = 0x20,
    /// Prefrontal Lobe: Rules, habits, preferences, and executable skills
    Prefrontal = 0x30,
    /// Occipital Lobe: Shapes, colors, and multimodal visual signatures
    Occipital = 0x40,
}

impl BrainLobe {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x05 => Some(Self::Hipocampo),
            0x10 => Some(Self::Temporal),
            0x20 => Some(Self::Parietal),
            0x30 => Some(Self::Prefrontal),
            0x40 => Some(Self::Occipital),
            _ => None,
        }
    }

    pub fn as_byte(self) -> u8 {
        self as u8
    }

    pub fn dir_name(self) -> &'static str {
        match self {
            Self::Hipocampo => "hipocampo",
            Self::Temporal => "temporal",
            Self::Parietal => "parietal",
            Self::Prefrontal => "prefrontal",
            Self::Occipital => "occipital",
        }
    }

    pub fn default_subfile(self) -> &'static str {
        match self {
            Self::Hipocampo => "diario_vigilia.atena",
            Self::Temporal => "identidades.atena",
            Self::Parietal => "associacoes_locais.atena",
            Self::Prefrontal => "regras_preferencias.atena",
            Self::Occipital => "formas_visuals.atena",
        }
    }
}

/// Semantic node type in the graph
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum NodeType {
    /// Physical or conceptual object (e.g.: "Blue Pencil")
    Object = 0x01,
    /// Action or verb (e.g.: "Store")
    Action = 0x02,
    /// Location or container (e.g.: "Box 2")
    Container = 0x03,
    /// Attribute or property (e.g.: "Blue Color")
    Attribute = 0x04,
    /// Inhibitory Rule or Error Alert (e.g.: "Avoid using Python 2")
    RuleOrAlert = 0x05,
}

impl NodeType {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Self::Object),
            0x02 => Some(Self::Action),
            0x03 => Some(Self::Container),
            0x04 => Some(Self::Attribute),
            0x05 => Some(Self::RuleOrAlert),
            _ => None,
        }
    }

    pub fn as_byte(self) -> u8 {
        self as u8
    }

    /// Readable name of the node type
    pub fn name(&self) -> &'static str {
        match self {
            Self::Object => "Object/Entity",
            Self::Action => "Action",
            Self::Container => "Location/Container",
            Self::Attribute => "Attribute",
            Self::RuleOrAlert => "Rule/Alert",
        }
    }
}

/// Relationship type (edge) between nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum RelationType {
    /// STORED_WITH — object stored alongside another
    StoredWith = 0xA1,
    /// LOCATED_IN — object inside a container/location
    LocatedIn = 0xA2,
    /// HAS_PROPERTY — node possesses an attribute
    HasProperty = 0xA3,
    /// AVOID_ACTION — inhibitory safeguard relationship (learning from error)
    AvoidAction = 0xA4,
}

impl RelationType {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0xA1 => Some(Self::StoredWith),
            0xA2 => Some(Self::LocatedIn),
            0xA3 => Some(Self::HasProperty),
            0xA4 => Some(Self::AvoidAction),
            _ => None,
        }
    }

    pub fn as_byte(self) -> u8 {
        self as u8
    }

    /// Readable label for the LLM synthesizer (in English for universal model compatibility)
    pub fn label(&self) -> &'static str {
        match self {
            Self::StoredWith => "STORED_WITH",
            Self::LocatedIn => "LOCATED_IN",
            Self::HasProperty => "HAS_PROPERTY",
            Self::AvoidAction => "AVOID_ACTION",
        }
    }

    /// Friendly name for documentation and reports
    pub fn name(&self) -> &'static str {
        match self {
            Self::StoredWith => "STORED_WITH",
            Self::LocatedIn => "LOCATED_IN",
            Self::HasProperty => "HAS_PROPERTY",
            Self::AvoidAction => "AVOID_ACTION",
        }
    }
}

// =============================================================================
// Structs principais
// =============================================================================

/// File header for binary .atena file (15 fixed bytes)
#[derive(Debug, Clone)]
pub struct FileHeader {
    pub magic: [u8; 4],
    pub version: u16,
    pub compression_type: CompressionType,
    pub uncompressed_size: u64,
}

impl FileHeader {
    pub const SIZE: usize = 15; // 4 + 2 + 1 + 8

    /// Serializes header into exactly 15 bytes (little-endian)
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut buf = [0u8; Self::SIZE];
        buf[0..4].copy_from_slice(&self.magic);
        buf[4..6].copy_from_slice(&self.version.to_le_bytes());
        buf[6] = self.compression_type.as_byte();
        buf[7..15].copy_from_slice(&self.uncompressed_size.to_le_bytes());
        buf
    }

    /// Deserializes header from 15 bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() < Self::SIZE {
            return Err(format!(
                "Header incompleto: esperado {} bytes, recebido {}",
                Self::SIZE,
                data.len()
            ));
        }

        let magic: [u8; 4] = data[0..4].try_into().unwrap();
        if magic != MAGIC_BYTES {
            return Err(format!(
                "Magic bytes inválidos: {:?} (esperado: {:?})",
                magic, MAGIC_BYTES
            ));
        }

        let version = u16::from_le_bytes(data[4..6].try_into().unwrap());
        let compression_byte = data[6];
        let compression_type = CompressionType::from_byte(compression_byte).ok_or_else(|| {
            format!("Tipo de compressão desconhecido: 0x{:02X}", compression_byte)
        })?;
        let uncompressed_size = u64::from_le_bytes(data[7..15].try_into().unwrap());

        Ok(Self {
            magic,
            version,
            compression_type,
            uncompressed_size,
        })
    }
}

/// Associative graph node with Biological Valence, Timestamp, and Session Scope
///
/// Binary layout v1: id(4) + type_flag(1) + valence(1) + label_len(2) + label(N)
/// Binary layout v2: id(4) + type_flag(1) + valence(1) + created_at(8) + label_len(2) + label(N)
/// Binary layout v3: id(4) + type_flag(1) + valence(1) + created_at(8) + session_len(2) + session_id(S) + label_len(2) + label(N)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryNode {
    pub id: u32,
    pub type_flag: NodeType,
    pub label: String,
    #[serde(default)]
    pub valence: i8, // -1 = Negative/Error/Inhibition, 0 = Neutral, 1 = Positive/Reward
    #[serde(default)]
    pub created_at: u64, // Epoch seconds timestamp when node was created
    #[serde(default)]
    pub session_id: Option<String>, // None = Global Memory; Some(id) = Session Private
}

impl MemoryNode {
    /// Serializes node to bytes (little-endian, v3 format with session_id)
    pub fn to_bytes(&self) -> Vec<u8> {
        let label_bytes = self.label.as_bytes();
        let label_len = label_bytes.len() as u16;
        let session_bytes = self.session_id.as_deref().unwrap_or("").as_bytes();
        let session_len = session_bytes.len() as u16;

        let mut buf = Vec::with_capacity(4 + 1 + 1 + 8 + 2 + session_bytes.len() + 2 + label_bytes.len());

        buf.extend_from_slice(&self.id.to_le_bytes());
        buf.push(self.type_flag.as_byte());
        buf.push(self.valence as u8);
        buf.extend_from_slice(&self.created_at.to_le_bytes());
        buf.extend_from_slice(&session_len.to_le_bytes());
        buf.extend_from_slice(session_bytes);
        buf.extend_from_slice(&label_len.to_le_bytes());
        buf.extend_from_slice(label_bytes);

        buf
    }

    /// Deserializes a node from a slice using current version
    pub fn from_bytes(data: &[u8]) -> Result<(Self, usize), String> {
        Self::from_bytes_version(data, CURRENT_VERSION)
    }

    /// Deserializes a node from a slice with backwards compatibility support (v1, v2, and v3)
    pub fn from_bytes_version(data: &[u8], version: u16) -> Result<(Self, usize), String> {
        if version <= 1 {
            if data.len() < 7 {
                return Err("Dados insuficientes para deserializar MemoryNode v1".into());
            }

            let id = u32::from_le_bytes(data[0..4].try_into().unwrap());
            let type_flag = NodeType::from_byte(data[4])
                .ok_or_else(|| format!("NodeType desconhecido: 0x{:02X}", data[4]))?;

            let (valence, label_len, header_size) = if data.len() >= 8 {
                let val = data[5] as i8;
                let len = u16::from_le_bytes(data[6..8].try_into().unwrap()) as usize;
                (val, len, 8)
            } else {
                let len = u16::from_le_bytes(data[5..7].try_into().unwrap()) as usize;
                (0i8, len, 7)
            };

            if data.len() < header_size + label_len {
                return Err(format!(
                    "Dados insuficientes para label: precisa de {} bytes, disponível {}",
                    label_len,
                    data.len() - header_size
                ));
            }

            let label = String::from_utf8(data[header_size..header_size + label_len].to_vec())
                .map_err(|e| format!("Label UTF-8 inválida: {}", e))?;

            Ok((
                Self {
                    id,
                    type_flag,
                    label,
                    valence,
                    created_at: 0,
                    session_id: None,
                },
                header_size + label_len,
            ))
        } else if version == 2 {
            // Versão 2: id(4) + type(1) + valence(1) + created_at(8) + label_len(2) + label(N)
            const HEADER_V2: usize = 16;
            if data.len() < HEADER_V2 {
                return Err("Dados insuficientes para deserializar MemoryNode v2".into());
            }

            let id = u32::from_le_bytes(data[0..4].try_into().unwrap());
            let type_flag = NodeType::from_byte(data[4])
                .ok_or_else(|| format!("NodeType desconhecido: 0x{:02X}", data[4]))?;
            let valence = data[5] as i8;
            let created_at = u64::from_le_bytes(data[6..14].try_into().unwrap());
            let label_len = u16::from_le_bytes(data[14..16].try_into().unwrap()) as usize;

            if data.len() < HEADER_V2 + label_len {
                return Err(format!(
                    "Dados insuficientes para label v2: precisa de {} bytes, disponível {}",
                    label_len,
                    data.len() - HEADER_V2
                ));
            }

            let label = String::from_utf8(data[HEADER_V2..HEADER_V2 + label_len].to_vec())
                .map_err(|e| format!("Label UTF-8 inválida: {}", e))?;

            Ok((
                Self {
                    id,
                    type_flag,
                    label,
                    valence,
                    created_at,
                    session_id: None,
                },
                HEADER_V2 + label_len,
            ))
        } else {
            // Versão 3+: id(4) + type(1) + valence(1) + created_at(8) + session_len(2) + session_id(S) + label_len(2) + label(N)
            const HEADER_V3: usize = 16;
            if data.len() < HEADER_V3 {
                return Err("Dados insuficientes para deserializar MemoryNode v3".into());
            }

            let id = u32::from_le_bytes(data[0..4].try_into().unwrap());
            let type_flag = NodeType::from_byte(data[4])
                .ok_or_else(|| format!("NodeType desconhecido: 0x{:02X}", data[4]))?;
            let valence = data[5] as i8;
            let created_at = u64::from_le_bytes(data[6..14].try_into().unwrap());
            let session_len = u16::from_le_bytes(data[14..16].try_into().unwrap()) as usize;

            if data.len() < HEADER_V3 + session_len + 2 {
                return Err("Dados insuficientes para session_id e label_len v3".into());
            }

            let session_id = if session_len > 0 {
                let sid = String::from_utf8(data[HEADER_V3..HEADER_V3 + session_len].to_vec())
                    .map_err(|e| format!("SessionId UTF-8 inválido: {}", e))?;
                Some(sid)
            } else {
                None
            };

            let label_offset = HEADER_V3 + session_len;
            let label_len = u16::from_le_bytes(data[label_offset..label_offset + 2].try_into().unwrap()) as usize;
            let total_len = label_offset + 2 + label_len;

            if data.len() < total_len {
                return Err(format!(
                    "Dados insuficientes para label v3: precisa de {} bytes, disponível {}",
                    total_len,
                    data.len()
                ));
            }

            let label = String::from_utf8(data[label_offset + 2..total_len].to_vec())
                .map_err(|e| format!("Label UTF-8 inválida: {}", e))?;

            Ok((
                Self {
                    id,
                    type_flag,
                    label,
                    valence,
                    created_at,
                    session_id,
                },
                total_len,
            ))
        }
    }
}

/// Graph edge (connection) with synaptic plasticity and temporal tracking
///
/// Fixed binary layout v1: 25 bytes
///   source_id(4) + target_id(4) + relation_type(1) + weight(4) + access_count(4) + last_accessed(8)
/// Fixed binary layout v2: 33 bytes
///   source_id(4) + target_id(4) + relation_type(1) + weight(4) + access_count(4) + last_accessed(8) + created_at(8)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEdge {
    pub source_id: u32,
    pub target_id: u32,
    pub relation_type: RelationType,
    /// Association strength (synaptic plasticity) — starts at 1.0
    pub weight: f32,
    /// Access counter (repetition reinforcement)
    pub access_count: u32,
    /// Last accessed timestamp (epoch seconds)
    pub last_accessed: u64,
    /// Creation timestamp of connection/memory (epoch seconds)
    #[serde(default)]
    pub created_at: u64,
}

impl MemoryEdge {
    pub const BYTE_SIZE_V1: usize = 25; // 4 + 4 + 1 + 4 + 4 + 8
    pub const BYTE_SIZE: usize = 33;    // 4 + 4 + 1 + 4 + 4 + 8 + 8 (V2 with created_at)

    /// Serializes edge into exactly 33 bytes (little-endian, v2 format)
    pub fn to_bytes(&self) -> [u8; Self::BYTE_SIZE] {
        let mut buf = [0u8; Self::BYTE_SIZE];
        buf[0..4].copy_from_slice(&self.source_id.to_le_bytes());
        buf[4..8].copy_from_slice(&self.target_id.to_le_bytes());
        buf[8] = self.relation_type.as_byte();
        buf[9..13].copy_from_slice(&self.weight.to_le_bytes());
        buf[13..17].copy_from_slice(&self.access_count.to_le_bytes());
        buf[17..25].copy_from_slice(&self.last_accessed.to_le_bytes());
        let effective_created_at = if self.created_at > 0 { self.created_at } else { self.last_accessed };
        buf[25..33].copy_from_slice(&effective_created_at.to_le_bytes());
        buf
    }

    /// Deserializes edge using the current version
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        Self::from_bytes_version(data, CURRENT_VERSION)
    }

    /// Deserializes an edge from bytes with backwards compatibility for v1 (25 bytes) and v2 (33 bytes)
    pub fn from_bytes_version(data: &[u8], version: u16) -> Result<Self, String> {
        if version <= 1 || (data.len() >= Self::BYTE_SIZE_V1 && data.len() < Self::BYTE_SIZE) {
            if data.len() < Self::BYTE_SIZE_V1 {
                return Err(format!(
                    "Dados insuficientes para MemoryEdge v1: esperado {}, recebido {}",
                    Self::BYTE_SIZE_V1,
                    data.len()
                ));
            }

            let source_id = u32::from_le_bytes(data[0..4].try_into().unwrap());
            let target_id = u32::from_le_bytes(data[4..8].try_into().unwrap());
            let relation_type = RelationType::from_byte(data[8])
                .ok_or_else(|| format!("RelationType desconhecido: 0x{:02X}", data[8]))?;
            let weight = f32::from_le_bytes(data[9..13].try_into().unwrap());
            let access_count = u32::from_le_bytes(data[13..17].try_into().unwrap());
            let last_accessed = u64::from_le_bytes(data[17..25].try_into().unwrap());

            Ok(Self {
                source_id,
                target_id,
                relation_type,
                weight,
                access_count,
                last_accessed,
                created_at: last_accessed,
            })
        } else {
            if data.len() < Self::BYTE_SIZE {
                return Err(format!(
                    "Dados insuficientes para MemoryEdge v2: esperado {}, recebido {}",
                    Self::BYTE_SIZE,
                    data.len()
                ));
            }

            let source_id = u32::from_le_bytes(data[0..4].try_into().unwrap());
            let target_id = u32::from_le_bytes(data[4..8].try_into().unwrap());
            let relation_type = RelationType::from_byte(data[8])
                .ok_or_else(|| format!("RelationType desconhecido: 0x{:02X}", data[8]))?;
            let weight = f32::from_le_bytes(data[9..13].try_into().unwrap());
            let access_count = u32::from_le_bytes(data[13..17].try_into().unwrap());
            let last_accessed = u64::from_le_bytes(data[17..25].try_into().unwrap());
            let created_at_raw = u64::from_le_bytes(data[25..33].try_into().unwrap());
            let created_at = if created_at_raw > 0 { created_at_raw } else { last_accessed };

            Ok(Self {
                source_id,
                target_id,
                relation_type,
                weight,
                access_count,
                last_accessed,
                created_at,
            })
        }
    }
}

/// Result of an associative sweep — path traversed across the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociationStep {
    pub node_label: String,
    pub node_type: NodeType,
    pub relation: Option<RelationType>,
    pub edge_weight: Option<f32>,
    #[serde(default)]
    pub edge_timestamp: Option<u64>,
}


/// Complete association pathway returned by Spreading Activation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociationPath {
    pub steps: Vec<AssociationStep>,
    /// Accumulated path weight (product of edge weights)
    pub total_weight: f32,
}

/// Event captured in the Hippocampal Lobe during waking hours (daily log)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VigiliaEvent {
    pub id: u64,
    pub timestamp: u64,
    pub speaker: String,  // "user" | "atena" | "system"
    pub text: String,
    pub valence: i8,      // -1 (Error/Aversion), 0 (Neutral), +1 (Reward/Success)
    pub salience: f32,    // 0.0 to 1.0 (relevance)
    pub category: String, // "dialogue" | "fact" | "preference" | "error_alert" | "action"
}

/// Consolidation and pruning report generated during sleep phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepConsolidationReport {
    pub timestamp: u64,
    pub events_processed: usize,
    pub noise_discarded: usize,
    pub facts_consolidated: usize,
    pub rules_created: usize,
    pub synapses_reinforced: usize,
    pub synapses_pruned: usize,
    pub positive_count: usize,
    pub negative_count: usize,
    pub details: Vec<String>,
}

/// Report on structural graph optimization, connection analysis, and pruning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphOptimizationReport {
    pub timestamp: u64,
    pub nodes_before: usize,
    pub nodes_after: usize,
    pub edges_before: usize,
    pub edges_after: usize,
    pub corrupted_removed: usize,
    pub duplicates_removed: usize,
    pub synapses_pruned: usize,
    pub orphan_nodes_pruned: usize,
    pub details: Vec<String>,
}

/// Single execution step of a procedural skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillStep {
    pub order: u32,
    pub instruction: String,
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub script_file: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

impl SkillStep {
    /// Returns the execution command, falling back to any command enclosed in backticks within instruction
    pub fn effective_command(&self) -> Option<String> {
        if let Some(ref c) = self.command {
            let tr = c.trim();
            if !tr.is_empty() {
                return Some(tr.to_string());
            }
        }
        if let Some(start) = self.instruction.find('`') {
            if let Some(end) = self.instruction[start + 1..].find('`') {
                let cmd = self.instruction[start + 1..start + 1 + end].trim();
                if !cmd.is_empty() {
                    return Some(cmd.to_string());
                }
            }
        }
        None
    }
}

/// Procedural skill learned, stored, and refined by Atena
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProceduralSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub triggers: Vec<String>,
    pub steps: Vec<SkillStep>,
    pub version: u32,
    pub executions_count: u32,
    pub success_count: u32,
    pub last_refined_at: u64,
    pub refinement_notes: Vec<String>,
    #[serde(default)]
    pub folder_path: Option<String>,
    #[serde(default)]
    pub scripts: Vec<String>,
    #[serde(default)]
    pub env_vars: Option<std::collections::HashMap<String, String>>,
    #[serde(default = "default_skill_permission_mode")]
    pub permission_mode: String,
}

fn default_skill_permission_mode() -> String {
    "ask".to_string()
}

/// Execution result for a skill command or script
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCommandResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

/// Payload for creating or updating a script file within a skill's scripts/ directory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillScriptFilePayload {
    pub filename: String,
    pub content: String,
}


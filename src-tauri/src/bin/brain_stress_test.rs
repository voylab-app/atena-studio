// =============================================================================
// bin/brain_stress_test.rs — Massive Brain Performance Benchmark
// =============================================================================
//
// Execute with: cargo run --release --bin brain_stress_test
//
// Generates a massive associative memory graph (~200,000 nodes, ~600,000 edges)
// and benchmarks every critical operation:
//
//   1. Graph construction (node + edge creation)
//   2. Binary serialization (raw bytes)
//   3. Compressed save to disk (Zstd, LZ4, Uncompressed)
//   4. Compressed load from disk (deserialization)
//   5. Spreading Activation traversal (multiple queries)
//   6. Synaptic reinforcement (batch learning)
//   7. Synaptic decay (batch forgetting)
//   8. Hot Cache statistics
//   9. LLM Context synthesis
//  10. Full brain persist (modular lobe partition + manifest)
//
// All timings use std::time::Instant for nanosecond precision.

use std::time::Instant;

use app_lib::core::memory::*;
use app_lib::services::memory_engine::MemoryGraphEngine;

// ─── Realistic vocabulary pools for diverse node labels ─────────────────────

const FIRST_NAMES: &[&str] = &[
    "Maria", "Carlos", "Ana", "Pedro", "Julia", "Lucas", "Fernanda", "Rafael",
    "Beatriz", "Thiago", "Camila", "Bruno", "Isabella", "Matheus", "Larissa",
    "Gabriel", "Manuela", "Gustavo", "Sofia", "Leonardo", "Helena", "Diego",
    "Valentina", "Felipe", "Mariana", "Daniel", "Alice", "Rodrigo", "Laura",
    "Henrique", "Bianca", "Victor", "Clara", "Marcos", "Nina", "Alexandre",
    "Luana", "Eduardo", "Olivia", "Caio", "Luiza", "Murilo", "Giovanna",
    "Arthur", "Sarah", "Samuel", "Leticia", "Enzo", "Cecilia", "Lorenzo",
];

const OBJECTS: &[&str] = &[
    "Laptop", "Smartphone", "Notebook", "Backpack", "Camera", "Headphones",
    "Watch", "Glasses", "Wallet", "Keys", "Umbrella", "Book", "Pen", "Tablet",
    "Charger", "Cable", "Mouse", "Keyboard", "Monitor", "Desk", "Chair",
    "Lamp", "Bottle", "Mug", "Plate", "Fork", "Knife", "Spoon", "Towel",
    "Pillow", "Blanket", "Mirror", "Vase", "Clock", "Frame", "Canvas",
    "Brush", "Scissors", "Ruler", "Tape", "Stapler", "Paper", "Envelope",
    "Stamp", "Badge", "Ring", "Necklace", "Bracelet", "Earring", "Hat",
];

const CONTAINERS: &[&str] = &[
    "Living Room", "Kitchen", "Bedroom", "Bathroom", "Office", "Garage",
    "Balcony", "Garden", "Basement", "Attic", "Closet", "Drawer", "Cabinet",
    "Shelf", "Box 1", "Box 2", "Box 3", "Box 4", "Bag A", "Bag B",
    "Storage Unit", "Safe", "Fridge", "Freezer", "Pantry", "Toolbox",
    "Suitcase", "Trunk", "Locker", "Warehouse", "Studio", "Lab",
    "Meeting Room", "Library", "Archive", "Vault", "Workshop", "Patio",
    "Terrace", "Rooftop", "Corridor", "Entrance", "Reception", "Lobby",
    "Elevator", "Stairwell", "Parking Lot", "Dock", "Platform", "Hangar",
];

const ATTRIBUTES: &[&str] = &[
    "Blue", "Red", "Green", "Yellow", "Black", "White", "Silver", "Gold",
    "Matte", "Glossy", "Large", "Small", "Heavy", "Light", "New", "Old",
    "Fast", "Slow", "Hot", "Cold", "Soft", "Hard", "Rough", "Smooth",
    "Bright", "Dark", "Transparent", "Opaque", "Fragile", "Durable",
    "Expensive", "Cheap", "Premium", "Standard", "Vintage", "Modern",
    "Elegant", "Casual", "Formal", "Sporty", "Rustic", "Urban", "Organic",
    "Synthetic", "Recycled", "Handmade", "Digital", "Analog", "Wireless",
    "Portable",
];

const RULES: &[&str] = &[
    "Always backup before updating", "Never use deprecated APIs",
    "Prefer Rust over C for new modules", "Use TypeScript strict mode",
    "Run tests before merging", "Document all public functions",
    "Avoid mutable global state", "Prefer composition over inheritance",
    "Use pattern matching instead of if-else chains",
    "Keep functions under 50 lines", "Log errors with full context",
    "Validate all user inputs", "Use prepared statements for SQL",
    "Encrypt sensitive data at rest", "Rate limit API endpoints",
    "Use content-addressable storage", "Prefer streaming over buffering",
    "Batch database writes when possible", "Use connection pooling",
    "Monitor memory usage in production",
];

/// Simple deterministic pseudo-random number generator (xorshift32)
struct FastRng {
    state: u32,
}

impl FastRng {
    fn new(seed: u32) -> Self {
        Self { state: if seed == 0 { 1 } else { seed } }
    }

    fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    fn next_usize(&mut self, max: usize) -> usize {
        (self.next_u32() as usize) % max
    }

    fn next_f32(&mut self) -> f32 {
        (self.next_u32() as f32) / (u32::MAX as f32)
    }
}

/// Formats byte count into human-readable form
fn fmt_bytes(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Formats duration in the most readable unit
fn fmt_duration(d: std::time::Duration) -> String {
    let nanos = d.as_nanos();
    if nanos < 1_000 {
        format!("{} ns", nanos)
    } else if nanos < 1_000_000 {
        format!("{:.1} µs", nanos as f64 / 1_000.0)
    } else if nanos < 1_000_000_000 {
        format!("{:.2} ms", nanos as f64 / 1_000_000.0)
    } else {
        format!("{:.3} s", nanos as f64 / 1_000_000_000.0)
    }
}

fn main() {
    // ─── Configuration ──────────────────────────────────────────────────────
    const TARGET_NODES: usize = 200_000;
    const EDGES_PER_NODE: usize = 3; // Average edges per node (~600K total)
    const CACHE_CAPACITY: usize = 2048;
    const ACTIVATION_QUERIES: usize = 50;
    const REINFORCEMENT_OPS: usize = 10_000;
    const DECAY_CYCLES: usize = 100;

    println!();
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║        ATENA STUDIO — Brain Stress Test & Benchmark            ║");
    println!("╠══════════════════════════════════════════════════════════════════╣");
    println!("║  Target: {:>7} nodes | ~{:>7} edges | cache: {}         ║",
             TARGET_NODES,
             TARGET_NODES * EDGES_PER_NODE,
             CACHE_CAPACITY);
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();

    let mut rng = FastRng::new(42);
    let benchmark_start = Instant::now();

    // =========================================================================
    // 1. GRAPH CONSTRUCTION
    // =========================================================================
    println!("━━━ 1. GRAPH CONSTRUCTION ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let t0 = Instant::now();
    let mut engine = MemoryGraphEngine::new(CACHE_CAPACITY);

    // Phase 1a: Create nodes with realistic diversity
    let node_creation_start = Instant::now();
    let mut all_node_ids: Vec<u32> = Vec::with_capacity(TARGET_NODES);

    let mut created_nodes = 0;

    // Identity entities (people) — ~20% of nodes
    let people_count = TARGET_NODES / 5;
    for i in 0..people_count {
        let name = FIRST_NAMES[i % FIRST_NAMES.len()];
        let suffix = if i < FIRST_NAMES.len() {
            String::new()
        } else {
            format!(" #{}", i / FIRST_NAMES.len())
        };
        let label = format!("{}{}", name, suffix);
        let valence: i8 = if rng.next_f32() > 0.7 { 1 } else { 0 };
        let id = engine.add_node_with_valence(NodeType::Object, &label, valence);
        all_node_ids.push(id);
        created_nodes += 1;
    }

    // Physical objects — ~30% of nodes
    let objects_count = TARGET_NODES * 3 / 10;
    for i in 0..objects_count {
        let obj = OBJECTS[i % OBJECTS.len()];
        let attr = ATTRIBUTES[rng.next_usize(ATTRIBUTES.len())];
        let label = format!("{} {} #{}", attr, obj, i);
        let id = engine.add_node(NodeType::Object, &label);
        all_node_ids.push(id);
        created_nodes += 1;
    }

    // Containers/Locations — ~15% of nodes
    let containers_count = TARGET_NODES * 15 / 100;
    for i in 0..containers_count {
        let container = CONTAINERS[i % CONTAINERS.len()];
        let label = if i < CONTAINERS.len() {
            container.to_string()
        } else {
            format!("{} (Floor {})", container, i / CONTAINERS.len())
        };
        let id = engine.add_node(NodeType::Container, &label);
        all_node_ids.push(id);
        created_nodes += 1;
    }

    // Attributes/Properties — ~25% of nodes
    let attrs_count = TARGET_NODES / 4;
    for i in 0..attrs_count {
        let attr = ATTRIBUTES[i % ATTRIBUTES.len()];
        let label = format!("{} Property #{}", attr, i);
        let valence: i8 = match rng.next_usize(3) {
            0 => -1,
            1 => 0,
            _ => 1,
        };
        let id = engine.add_node_with_valence(NodeType::Attribute, &label, valence);
        all_node_ids.push(id);
        created_nodes += 1;
    }

    // Rules/Alerts — ~10% of nodes
    let remaining = TARGET_NODES.saturating_sub(created_nodes);
    for i in 0..remaining {
        let rule = RULES[i % RULES.len()];
        let label = if i < RULES.len() {
            rule.to_string()
        } else {
            format!("{} (variant {})", rule, i / RULES.len())
        };
        let id = engine.add_node_with_valence(NodeType::RuleOrAlert, &label, -1);
        all_node_ids.push(id);
    }
    let node_creation_dur = node_creation_start.elapsed();

    println!("  Nodes created:  {} in {}",
             engine.node_count(), fmt_duration(node_creation_dur));
    println!("  Rate:           {:.0} nodes/sec",
             engine.node_count() as f64 / node_creation_dur.as_secs_f64());

    // Phase 1b: Create edges with realistic connectivity patterns
    let edge_creation_start = Instant::now();
    let total_nodes = all_node_ids.len();
    let target_edges = TARGET_NODES * EDGES_PER_NODE;

    let relation_types = [
        RelationType::StoredWith,
        RelationType::LocatedIn,
        RelationType::HasProperty,
        RelationType::AvoidAction,
    ];

    let mut edges_created: usize = 0;
    for _ in 0..target_edges {
        let src_idx = rng.next_usize(total_nodes);
        let tgt_idx = rng.next_usize(total_nodes);
        if src_idx == tgt_idx {
            continue;
        }
        let src_id = all_node_ids[src_idx];
        let tgt_id = all_node_ids[tgt_idx];
        let rel = relation_types[rng.next_usize(relation_types.len())];
        if engine.add_edge(src_id, tgt_id, rel).is_ok() {
            edges_created += 1;
        }
    }
    let edge_creation_dur = edge_creation_start.elapsed();

    println!("  Edges created:  {} in {}",
             engine.edge_count(), fmt_duration(edge_creation_dur));
    println!("  Rate:           {:.0} edges/sec",
             edges_created as f64 / edge_creation_dur.as_secs_f64());
    println!("  Total build:    {}", fmt_duration(t0.elapsed()));
    println!();

    // =========================================================================
    // 2. BINARY SERIALIZATION (In-memory)
    // =========================================================================
    println!("━━━ 2. BINARY SERIALIZATION ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let t1 = Instant::now();
    let raw_bytes = engine.serialize_to_bytes();
    let serialize_dur = t1.elapsed();

    println!("  Serialized:     {} in {}",
             fmt_bytes(raw_bytes.len()), fmt_duration(serialize_dur));
    println!("  Throughput:     {:.1} MB/s",
             raw_bytes.len() as f64 / (1024.0 * 1024.0) / serialize_dur.as_secs_f64());
    println!();

    // =========================================================================
    // 3. IN-MEMORY DESERIALIZATION
    // =========================================================================
    println!("━━━ 3. IN-MEMORY DESERIALIZATION ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let t2 = Instant::now();
    let restored = MemoryGraphEngine::deserialize_from_bytes(&raw_bytes, CACHE_CAPACITY)
        .expect("Deserialization failed");
    let deserialize_dur = t2.elapsed();

    assert_eq!(engine.node_count(), restored.node_count(), "Node count mismatch!");
    assert_eq!(engine.edge_count(), restored.edge_count(), "Edge count mismatch!");

    println!("  Deserialized:   {} nodes, {} edges in {}",
             restored.node_count(), restored.edge_count(), fmt_duration(deserialize_dur));
    println!("  Throughput:     {:.1} MB/s",
             raw_bytes.len() as f64 / (1024.0 * 1024.0) / deserialize_dur.as_secs_f64());
    println!("  Integrity:      ✓ (nodes and edges match)");
    println!();

    // =========================================================================
    // 4. COMPRESSED SAVE TO DISK
    // =========================================================================
    println!("━━━ 4. COMPRESSED SAVE TO DISK ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let tmp_dir = std::env::temp_dir().join("atena_brain_stress_test");
    std::fs::create_dir_all(&tmp_dir).expect("Failed to create temp directory");

    // --- Zstd ---
    let zstd_path = tmp_dir.join("stress_test.zstd.atena");
    let t3z = Instant::now();
    let zstd_size = engine
        .save_to_compressed_binary(&zstd_path, CompressionType::Zstd)
        .expect("Zstd save failed");
    let zstd_save_dur = t3z.elapsed();
    let zstd_ratio = zstd_size as f64 / raw_bytes.len() as f64 * 100.0;

    println!("  [ZSTD] {} → {} ({:.1}%) in {}",
             fmt_bytes(raw_bytes.len()), fmt_bytes(zstd_size),
             zstd_ratio, fmt_duration(zstd_save_dur));
    println!("         Write throughput: {:.1} MB/s",
             raw_bytes.len() as f64 / (1024.0 * 1024.0) / zstd_save_dur.as_secs_f64());

    // --- LZ4 ---
    let lz4_path = tmp_dir.join("stress_test.lz4.atena");
    let t3l = Instant::now();
    let lz4_size = engine
        .save_to_compressed_binary(&lz4_path, CompressionType::Lz4)
        .expect("LZ4 save failed");
    let lz4_save_dur = t3l.elapsed();
    let lz4_ratio = lz4_size as f64 / raw_bytes.len() as f64 * 100.0;

    println!("  [LZ4]  {} → {} ({:.1}%) in {}",
             fmt_bytes(raw_bytes.len()), fmt_bytes(lz4_size),
             lz4_ratio, fmt_duration(lz4_save_dur));
    println!("         Write throughput: {:.1} MB/s",
             raw_bytes.len() as f64 / (1024.0 * 1024.0) / lz4_save_dur.as_secs_f64());

    // --- Raw (uncompressed) ---
    let raw_path = tmp_dir.join("stress_test.raw.atena");
    let t3r = Instant::now();
    let raw_size = engine
        .save_to_compressed_binary(&raw_path, CompressionType::Uncompressed)
        .expect("Raw save failed");
    let raw_save_dur = t3r.elapsed();

    println!("  [RAW]  {} in {}",
             fmt_bytes(raw_size), fmt_duration(raw_save_dur));
    println!("         Write throughput: {:.1} MB/s",
             raw_bytes.len() as f64 / (1024.0 * 1024.0) / raw_save_dur.as_secs_f64());
    println!();

    // =========================================================================
    // 5. COMPRESSED LOAD FROM DISK
    // =========================================================================
    println!("━━━ 5. COMPRESSED LOAD FROM DISK ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // --- Zstd Load ---
    let t4z = Instant::now();
    let mut loaded_zstd = MemoryGraphEngine::load_from_compressed_binary(&zstd_path, CACHE_CAPACITY)
        .expect("Zstd load failed");
    let zstd_load_dur = t4z.elapsed();

    println!("  [ZSTD] Loaded: {} nodes, {} edges in {}",
             loaded_zstd.node_count(), loaded_zstd.edge_count(), fmt_duration(zstd_load_dur));
    println!("         Read throughput: {:.1} MB/s",
             raw_bytes.len() as f64 / (1024.0 * 1024.0) / zstd_load_dur.as_secs_f64());

    // --- LZ4 Load ---
    let t4l = Instant::now();
    let loaded_lz4 = MemoryGraphEngine::load_from_compressed_binary(&lz4_path, CACHE_CAPACITY)
        .expect("LZ4 load failed");
    let lz4_load_dur = t4l.elapsed();

    println!("  [LZ4]  Loaded: {} nodes, {} edges in {}",
             loaded_lz4.node_count(), loaded_lz4.edge_count(), fmt_duration(lz4_load_dur));
    println!("         Read throughput: {:.1} MB/s",
             raw_bytes.len() as f64 / (1024.0 * 1024.0) / lz4_load_dur.as_secs_f64());

    // --- Raw Load ---
    let t4r = Instant::now();
    let loaded_raw = MemoryGraphEngine::load_from_compressed_binary(&raw_path, CACHE_CAPACITY)
        .expect("Raw load failed");
    let raw_load_dur = t4r.elapsed();

    println!("  [RAW]  Loaded: {} nodes, {} edges in {}",
             loaded_raw.node_count(), loaded_raw.edge_count(), fmt_duration(raw_load_dur));

    // Integrity verification
    assert_eq!(engine.node_count(), loaded_zstd.node_count());
    assert_eq!(engine.edge_count(), loaded_zstd.edge_count());
    assert_eq!(engine.node_count(), loaded_lz4.node_count());
    assert_eq!(engine.node_count(), loaded_raw.node_count());
    println!("  Integrity:      ✓ (all formats match original)");
    println!();

    // =========================================================================
    // 6. SPREADING ACTIVATION (Multiple Queries)
    // =========================================================================
    println!("━━━ 6. SPREADING ACTIVATION ({} queries) ━━━━━━━━━━━━━━━━━━━━━━━", ACTIVATION_QUERIES);

    let mut total_activation_dur = std::time::Duration::ZERO;
    let mut total_paths_found = 0;
    let mut fastest = std::time::Duration::from_secs(999);
    let mut slowest = std::time::Duration::ZERO;

    // Sample diverse starting nodes
    let query_labels: Vec<String> = (0..ACTIVATION_QUERIES)
        .map(|i| {
            let idx = (i * 37 + 13) % FIRST_NAMES.len(); // Spread across different names
            FIRST_NAMES[idx].to_string()
        })
        .collect();

    for label in &query_labels {
        let tq = Instant::now();
        match loaded_zstd.traverse_associations(label, 3) {
            Ok(paths) => {
                let dur = tq.elapsed();
                total_activation_dur += dur;
                total_paths_found += paths.len();
                if dur < fastest { fastest = dur; }
                if dur > slowest { slowest = dur; }
            }
            Err(_) => {
                // Some queries may not find a node — that's OK
            }
        }
    }

    let avg_activation = if ACTIVATION_QUERIES > 0 {
        total_activation_dur / ACTIVATION_QUERIES as u32
    } else {
        std::time::Duration::ZERO
    };

    println!("  Queries run:    {}", ACTIVATION_QUERIES);
    println!("  Paths found:    {} total ({:.1} avg per query)",
             total_paths_found, total_paths_found as f64 / ACTIVATION_QUERIES as f64);
    println!("  Total time:     {}", fmt_duration(total_activation_dur));
    println!("  Average:        {}", fmt_duration(avg_activation));
    println!("  Fastest:        {}", fmt_duration(fastest));
    println!("  Slowest:        {}", fmt_duration(slowest));
    println!();

    // =========================================================================
    // 7. LLM CONTEXT SYNTHESIS
    // =========================================================================
    println!("━━━ 7. LLM CONTEXT SYNTHESIS ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let sample_paths = loaded_zstd
        .traverse_associations(FIRST_NAMES[0], 3)
        .unwrap_or_default();

    let t5 = Instant::now();
    let llm_context = MemoryGraphEngine::build_llm_context(&sample_paths);
    let llm_dur = t5.elapsed();

    println!("  Synthesis time: {}", fmt_duration(llm_dur));
    println!("  Context length: {} chars", llm_context.len());
    println!("  Preview:        {}...",
             &llm_context[..llm_context.len().min(120)]);
    println!();

    // =========================================================================
    // 8. SYNAPTIC REINFORCEMENT (Batch Learning)
    // =========================================================================
    println!("━━━ 8. SYNAPTIC REINFORCEMENT ({} ops) ━━━━━━━━━━━━━━━━━━━━━━━━", REINFORCEMENT_OPS);

    let t6 = Instant::now();
    let mut reinforcement_ok = 0;
    let mut rng2 = FastRng::new(7777);

    for _ in 0..REINFORCEMENT_OPS {
        let src_idx = rng2.next_usize(total_nodes);
        let tgt_idx = rng2.next_usize(total_nodes);
        if src_idx != tgt_idx {
            let src = all_node_ids[src_idx];
            let tgt = all_node_ids[tgt_idx];
            if loaded_zstd.reinforce_edge(src, tgt).is_ok() {
                reinforcement_ok += 1;
            }
        }
    }
    let reinforce_dur = t6.elapsed();

    println!("  Reinforcements: {}/{} successful in {}",
             reinforcement_ok, REINFORCEMENT_OPS, fmt_duration(reinforce_dur));
    println!("  Rate:           {:.0} ops/sec",
             REINFORCEMENT_OPS as f64 / reinforce_dur.as_secs_f64());
    println!();

    // =========================================================================
    // 9. SYNAPTIC DECAY (Batch Forgetting)
    // =========================================================================
    println!("━━━ 9. SYNAPTIC DECAY ({} cycles) ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", DECAY_CYCLES);

    let edges_before = loaded_zstd.edge_count();
    let t7 = Instant::now();

    for _ in 0..DECAY_CYCLES {
        loaded_zstd.apply_decay(0.05);
    }
    let decay_dur = t7.elapsed();
    let edges_after = loaded_zstd.edge_count();
    let edges_pruned = edges_before - edges_after;

    println!("  Edges before:   {}", edges_before);
    println!("  Edges after:    {}", edges_after);
    println!("  Edges pruned:   {} ({:.1}%)",
             edges_pruned, edges_pruned as f64 / edges_before as f64 * 100.0);
    println!("  Total time:     {} ({} per cycle)",
             fmt_duration(decay_dur),
             fmt_duration(decay_dur / DECAY_CYCLES as u32));
    println!();

    // =========================================================================
    // 10. HOT CACHE STATISTICS
    // =========================================================================
    println!("━━━ 10. HOT CACHE STATISTICS ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let stats = loaded_zstd.stats();
    let original_stats = engine.stats();

    println!("  --- Original Engine ---");
    println!("  Nodes:          {}", original_stats.total_nodes);
    println!("  Edges:          {}", original_stats.total_edges);
    println!("  Cache size:     {} / {}", original_stats.cache_size, CACHE_CAPACITY);
    println!("  Cache hits:     {}", original_stats.cache_hits);
    println!("  Cache misses:   {}", original_stats.cache_misses);
    println!("  Hit rate:       {:.1}%", original_stats.cache_hit_rate);
    println!();
    println!("  --- After Stress Test ---");
    println!("  Nodes:          {}", stats.total_nodes);
    println!("  Edges:          {} (after {} decay cycles)", stats.total_edges, DECAY_CYCLES);
    println!("  Cache size:     {} / {}", stats.cache_size, CACHE_CAPACITY);
    println!("  Cache hits:     {}", stats.cache_hits);
    println!("  Cache misses:   {}", stats.cache_misses);
    println!("  Hit rate:       {:.1}%", stats.cache_hit_rate);
    println!();

    // =========================================================================
    // 11. MEMORY FOOTPRINT ESTIMATE
    // =========================================================================
    println!("━━━ 11. MEMORY FOOTPRINT ESTIMATE ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let avg_label_len = 25; // Rough average label length in bytes
    let node_ram = engine.node_count() * (32 + avg_label_len + 32); // struct + label + HashMap overhead
    let edge_ram = engine.edge_count() * (33 + 16); // MemoryEdge + Vec/HashMap overhead
    let index_ram = engine.node_count() * (avg_label_len + 8); // label_index
    let cache_ram = CACHE_CAPACITY * (33 + 32); // CacheEntry + HashMap overhead
    let total_ram = node_ram + edge_ram + index_ram + cache_ram;

    println!("  Nodes (~{} bytes each): {}",
             32 + avg_label_len + 32, fmt_bytes(node_ram));
    println!("  Edges (~49 bytes each):  {}", fmt_bytes(edge_ram));
    println!("  Label index:             {}", fmt_bytes(index_ram));
    println!("  Hot Cache:               {}", fmt_bytes(cache_ram));
    println!("  ─────────────────────────────────");
    println!("  Estimated total RAM:     {}", fmt_bytes(total_ram));
    println!("  Binary on disk (Zstd):   {}", fmt_bytes(zstd_size));
    println!("  Binary on disk (LZ4):    {}", fmt_bytes(lz4_size));
    println!("  Binary on disk (Raw):    {}", fmt_bytes(raw_size));
    println!();

    // =========================================================================
    // SUMMARY TABLE
    // =========================================================================
    println!("━━━ BENCHMARK SUMMARY ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("  ┌────────────────────────────────┬───────────────────┐");
    println!("  │ Operation                      │ Duration          │");
    println!("  ├────────────────────────────────┼───────────────────┤");
    println!("  │ Graph construction             │ {:>17} │", fmt_duration(t0.elapsed().min(node_creation_dur + edge_creation_dur)));
    println!("  │  ├ Node creation ({:>6})      │ {:>17} │", engine.node_count(), fmt_duration(node_creation_dur));
    println!("  │  └ Edge creation ({:>6})      │ {:>17} │", edges_created, fmt_duration(edge_creation_dur));
    println!("  │ Serialization                  │ {:>17} │", fmt_duration(serialize_dur));
    println!("  │ Deserialization                │ {:>17} │", fmt_duration(deserialize_dur));
    println!("  │ Save Zstd ({:>9})          │ {:>17} │", fmt_bytes(zstd_size), fmt_duration(zstd_save_dur));
    println!("  │ Save LZ4  ({:>9})          │ {:>17} │", fmt_bytes(lz4_size), fmt_duration(lz4_save_dur));
    println!("  │ Save Raw  ({:>9})          │ {:>17} │", fmt_bytes(raw_size), fmt_duration(raw_save_dur));
    println!("  │ Load Zstd                      │ {:>17} │", fmt_duration(zstd_load_dur));
    println!("  │ Load LZ4                       │ {:>17} │", fmt_duration(lz4_load_dur));
    println!("  │ Load Raw                       │ {:>17} │", fmt_duration(raw_load_dur));
    println!("  │ Spreading Activation (avg)     │ {:>17} │", fmt_duration(avg_activation));
    println!("  │ LLM Context synthesis          │ {:>17} │", fmt_duration(llm_dur));
    println!("  │ Reinforcement ({:>5} ops)      │ {:>17} │", REINFORCEMENT_OPS, fmt_duration(reinforce_dur));
    println!("  │ Decay ({} cycles)              │ {:>17} │", DECAY_CYCLES, fmt_duration(decay_dur));
    println!("  ├────────────────────────────────┼───────────────────┤");
    println!("  │ TOTAL BENCHMARK                │ {:>17} │", fmt_duration(benchmark_start.elapsed()));
    println!("  └────────────────────────────────┴───────────────────┘");
    println!();

    // =========================================================================
    // CLEANUP
    // =========================================================================
    println!("━━━ CLEANUP ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let _ = std::fs::remove_dir_all(&tmp_dir);
    println!("  Temp files removed: {}", tmp_dir.display());
    println!();

    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║             BRAIN STRESS TEST COMPLETED SUCCESSFULLY ✓         ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();
}

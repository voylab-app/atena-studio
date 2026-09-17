use app_lib::core::memory::*;
use app_lib::services::memory_engine::*;

#[test]
fn test_massive_brain_population_and_multi_hop_traversal() {
    let mut engine = MemoryGraphEngine::new(256);
    let now = MemoryGraphEngine::current_timestamp();

    // 1. User Identity, Preferences & Work (Neutral fictitious user setup)
    let user_facts = [
        ("User", "Role: Systems Architect", 1),
        ("User", "Primary Language: Rust", 1),
        ("User", "Secondary Language: TypeScript", 1),
        ("User", "Editor: Neovim", 1),
        ("User", "Theme: Dark Mode", 1),
        ("User", "Prefers mechanical keyboards", 1),
        ("User", "Drinks pour-over coffee", 1),
        ("User", "Working on: Project Atena", 1),
        ("User", "Timezone: UTC-3", 1),
    ];
    for (s, p, v) in user_facts {
        engine.insert_declarative_fact(s, p, v, None, now - 5000);
    }

    // 2. Social Network & Entities (Using neutral fictitious names per AGENTS.md)
    let people_facts = [
        ("Maria", "Friend of User", 1),
        ("Maria", "Profession: Architect", 1),
        ("Maria", "Spouse: Carlos", 1),
        ("Maria", "Child: Luna", 1),
        ("Maria", "Favorite Style: Minimalist Scandinavian", 1),
        ("Maria", "Wedding Date: 14/11/2026", 1),
        ("Maria", "Wedding Gift: Smart Air Fryer Pro", 1),
        ("Carlos", "Husband of Maria", 1),
        ("Carlos", "Father of Luna", 1),
        ("Carlos", "Profession: Software Engineer", 1),
        ("Carlos", "Hair: Dark Brown", 1),
        ("Carlos", "Eyes: Amber Brown", 1),
        ("Carlos", "Dental: Wears Ceramic Braces", 1),
        ("Luna", "Daughter of Maria", 1),
        ("Luna", "Daughter of Carlos", 1),
        ("Luna", "Age: 2 years old", 1),
        ("Luna", "Favorite Toy: Wooden Building Blocks", 1),
        ("Ana", "Colleague of User", 1),
        ("Ana", "Role: Lead ML Research Scientist", 1),
        ("Ana", "Specialty: Graph Neural Networks", 1),
        ("Ana", "Birthday: 22/05/1995", 1),
        ("Lucas", "Colleague of User", 1),
        ("Lucas", "Role: Infrastructure Specialist", 1),
        ("Lucas", "Prefers morning focus sessions", 1),
        ("Sofia", "Designer of User", 1),
        ("Sofia", "Role: Principal UI/UX Designer", 1),
        ("Sofia", "Design Tool: Figma", 1),
        ("Pedro", "Friend of User", 1),
        ("Pedro", "Role: Systems Developer", 1),
        ("Elena", "Mentor of User", 1),
        ("Elena", "Author of Concurrent Computing Handbook", 1),
    ];
    for (s, p, v) in people_facts {
        engine.insert_declarative_fact(s, p, v, None, now - 3600);
    }

    // 3. Physical Devices, Gadgets & Hardware
    let devices = [
        ("MacBook Pro M3 Max", "Hardware: 64GB Unified RAM, 2TB SSD"),
        ("Apple Pro Display XDR", "Hardware: 32-inch 6K Retina"),
        ("Keychron Q1 Pro", "Hardware: Wireless 75% Custom Mechanical"),
        ("Logitech MX Master 3S", "Hardware: Ergonomic Wireless Mouse"),
        ("Sony WH-1000XM5", "Hardware: Noise-Cancelling Wireless Headphones"),
        ("iPad Pro 12.9", "Hardware: M2 Chip with Apple Pencil 2"),
        ("Kindle Paperwhite", "Hardware: Warm Light E-Reader"),
        ("iPhone 16 Pro", "Hardware: Titanium 256GB"),
        ("La Marzocco Micra", "Hardware: Dual Boiler Espresso Machine"),
        ("Comandante C40", "Hardware: Manual Coffee Grinder Nitro Blade"),
        ("Bose SoundLink Flex", "Hardware: Outdoor Bluetooth Speaker"),
        ("Sony A7 IV", "Hardware: Full-Frame Mirrorless Camera"),
        ("Sigma 24-70mm f2.8", "Hardware: Art Zoom Lens"),
        ("Peak Design Everyday Backpack", "Hardware: 20L Charcoal Backpack"),
        ("Yubikey 5C NFC", "Hardware: FIDO2 Hardware Security Key"),
    ];
    for (dev, spec) in devices {
        engine.insert_declarative_fact(dev, spec, 1, None, now - 2000);
    }

    // 4. Physical Spatial Containers & Location Hierarchy
    let locations = [
        ("Desk Drawer", "Inside Home Office"),
        ("Top Shelf", "Inside Home Office"),
        ("Safe Box", "Inside Master Bedroom Closet"),
        ("Kitchen Counter", "Inside Ground Floor Kitchen"),
        ("Pantry Cabinet", "Inside Ground Floor Kitchen"),
        ("Living Room Bookshelf", "Inside Living Room"),
        ("Toolbox A", "Inside Garage Workshop"),
        ("Backpack Main Compartment", "Inside Peak Design Everyday Backpack"),
    ];
    for (loc, container) in locations {
        engine.insert_container_fact(loc, container, None, now - 1500);
    }

    // 5. Item Placements (LocatedIn)
    let item_locations = [
        ("Passport", "Safe Box"),
        ("Backup Emergency Cash", "Safe Box"),
        ("Yubikey 5C NFC", "Desk Drawer"),
        ("Spare USB-C Cables", "Desk Drawer"),
        ("MacBook Pro M3 Max", "Home Office"),
        ("Apple Pro Display XDR", "Home Office"),
        ("Keychron Q1 Pro", "Home Office"),
        ("Sony WH-1000XM5", "Backpack Main Compartment"),
        ("iPad Pro 12.9", "Backpack Main Compartment"),
        ("Kindle Paperwhite", "Living Room Bookshelf"),
        ("La Marzocco Micra", "Kitchen Counter"),
        ("Comandante C40", "Kitchen Counter"),
        ("Roasted Coffee Beans", "Kitchen Counter"),
        ("Sony A7 IV", "Top Shelf"),
        ("Sigma 24-70mm f2.8", "Top Shelf"),
        ("House Keys", "Desk Drawer"),
        ("Car Key Fob", "Desk Drawer"),
    ];
    for (item, loc) in item_locations {
        engine.insert_container_fact(item, loc, None, now - 1000);
    }

    // 6. StoredWith Item Associations
    let stored_with_pairs = [
        ("MacBook Pro M3 Max", "Sony WH-1000XM5"),
        ("Sony A7 IV", "Sigma 24-70mm f2.8"),
        ("La Marzocco Micra", "Comandante C40"),
        ("Comandante C40", "Roasted Coffee Beans"),
        ("Passport", "Backup Emergency Cash"),
        ("House Keys", "Car Key Fob"),
    ];
    for (a, b) in stored_with_pairs {
        let aid = engine.get_or_create_node(NodeType::Object, a);
        let bid = engine.get_or_create_node(NodeType::Object, b);
        let _ = engine.add_edge_with_timestamp(aid, bid, RelationType::StoredWith, now - 800);
        let _ = engine.add_edge_with_timestamp(bid, aid, RelationType::StoredWith, now - 800);
    }

    // 7. Behavioral Rules, Inhibitions & Negative Alerts (valences = -1)
    let negative_rules = [
        ("User", "Rule: Never commit secret API keys or credentials to git"),
        ("User", "Rule: Avoid scheduling non-urgent meetings before 10 AM"),
        ("User", "Rule: Do not consume caffeinated drinks after 3 PM"),
        ("User", "Rule: Avoid loud noisy co-working spaces during focus blocks"),
        ("User", "Rule: Never push unreviewed code directly to main branch"),
        ("Carlos", "Rule: Avoid heavy carbs during lunch to prevent afternoon fatigue"),
        ("Maria", "Rule: Avoid artificial fluorescent lighting in living spaces"),
        ("Ana", "Rule: Never use float precision when calculating financial values"),
    ];
    for (subj, rule) in negative_rules {
        engine.insert_rule_fact(subj, rule, None, now - 600);
    }

    // 8. Projects & Research Initiatives
    let projects = [
        ("Project Atena", "Description: Cognitive AI Desktop Workspace with Local Neural Engine"),
        ("Project Atena", "Milestone: Public Beta Launch on 30/11/2026"),
        ("Project Atena", "Architecture: Zero-IPC Rust Core + Nitro Frontend"),
        ("Project Atena", "Inference: MLX on Apple Silicon + llama.cpp fallback"),
        ("Project Atena", "Memory Topology: Hexagonal 4-Lobe Associative Graph"),
        ("GGUF Engine", "Component of Project Atena"),
        ("MLX Backend", "Component of Project Atena"),
        ("MCP Protocol", "Component of Project Atena"),
    ];
    for (proj, detail) in projects {
        engine.insert_declarative_fact(proj, detail, 1, None, now - 400);
    }

    // Connect User and Collaborators to Project Atena
    let atena_id = engine.get_or_create_node(NodeType::Object, "Project Atena");
    let user_id = engine.find_node_by_label("User").unwrap().id;
    let ana_id = engine.find_node_by_label("Ana").unwrap().id;
    let lucas_id = engine.find_node_by_label("Lucas").unwrap().id;
    let sofia_id = engine.find_node_by_label("Sofia").unwrap().id;

    let _ = engine.add_edge_with_timestamp(user_id, atena_id, RelationType::HasProperty, now);
    let _ = engine.add_edge_with_timestamp(ana_id, atena_id, RelationType::HasProperty, now);
    let _ = engine.add_edge_with_timestamp(lucas_id, atena_id, RelationType::HasProperty, now);
    let _ = engine.add_edge_with_timestamp(sofia_id, atena_id, RelationType::HasProperty, now);

    // Verify massive brain scale
    println!("Total Brain Nodes: {}", engine.node_count());
    println!("Total Brain Edges: {}", engine.edge_count());
    assert!(engine.node_count() >= 75, "Expected at least 75 nodes, got {}", engine.node_count());
    assert!(engine.edge_count() >= 110, "Expected at least 110 edges, got {}", engine.edge_count());

    // 9. Verify Multi-Hop Spreading Activation
    // Search: "Where are the house keys stored?"
    let paths_keys = engine.traverse_associations("House Keys", 3).expect("Traversal must succeed");
    assert!(!paths_keys.is_empty(), "Should find association paths for House Keys");
    let ctx_keys = MemoryGraphEngine::build_llm_context(&paths_keys);
    println!("LLM Context for 'House Keys':\n{}", ctx_keys);
    assert!(ctx_keys.contains("Desk Drawer"));
    assert!(ctx_keys.contains("[saved at:"));

    // Search: "Who is Carlos and what is his relationship to Maria?"
    let paths_carlos = engine.traverse_associations("Carlos", 3).expect("Traversal must succeed");
    assert!(!paths_carlos.is_empty());
    let ctx_carlos = MemoryGraphEngine::build_llm_context(&paths_carlos);
    println!("LLM Context for 'Carlos':\n{}", ctx_carlos);
    assert!(ctx_carlos.contains("Maria"));
    assert!(ctx_carlos.contains("Luna"));

    // Search: "What hardware is kept in the backpack?"
    let query_backpack = engine.search_active_context_for_query("Where are my headphones and backpack?");
    assert!(query_backpack.is_some());
    let bp_str = query_backpack.unwrap();
    println!("Context for Backpack query:\n{}", bp_str);
    assert!(bp_str.contains("Backpack") || bp_str.contains("Headphones"));

    // 10. Verify Full Graph Persistence & Partitioning Across All 4 Lobes
    let persist_res = engine.auto_persist_default();
    assert!(persist_res.is_ok(), "Auto-persist must succeed: {:?}", persist_res);
    let total_bytes = persist_res.unwrap();
    assert!(total_bytes > 0, "Persisted bytes must be > 0");

    // 11. Verify Binary Serialization & Deserialization Integrity
    let bytes = engine.serialize_to_bytes();
    let restored = MemoryGraphEngine::deserialize_from_bytes(&bytes, 256).expect("Deserialization failed");
    assert_eq!(restored.node_count(), engine.node_count());
    assert_eq!(restored.edge_count(), engine.edge_count());

    // 12. Verify Markdown Export with Entities and Neural Connections
    let md = engine.export_graph_markdown();
    assert!(md.contains("# Associative Memory Graph — Atena Studio"));
    assert!(md.contains("## 🧠 Entities and Concepts (Nodes)"));
    assert!(md.contains("## ⚡ Neural Connections (Synapses)"));
    assert!(md.contains("Positive (+1)"));
    assert!(md.contains("Inhibition (-1)"));
    assert!(md.contains("Desk Drawer"));
    assert!(md.contains("La Marzocco Micra"));
}

#[test]
fn test_large_scale_autonomous_turn_ingestion() {
    let mut engine = MemoryGraphEngine::with_default_cache();

    let turns = [
        r#"Welcome! I have noted your initial workstation setup.
<memory subject="User" property="Workstation: Triple Monitor Setup" type="Object" valence="1" />
<memory subject="User" property="Chair: Ergonomic Mesh Chair" type="Object" valence="1" />
<memory subject="Desk" property="Located In: Office Room" type="Container" valence="0" />"#,

        r#"Added your preferred audio gear.
<memory subject="Headphones" property="Located In: Desk Drawer" type="Container" valence="0" />
<memory subject="Microphone" property="Model: Dynamic Studio Mic" type="Object" valence="1" />
<memory subject="Microphone" property="Located In: Desk" type="Container" valence="0" />"#,

        r#"Recorded meeting with colleague David.
<memory subject="User" property="Colleague: David" type="Object" valence="1" />
<memory subject="David" property="Role: Security Architect" type="Object" valence="1" />
<memory subject="David" property="Preference: Uses GPG Signing" type="Object" valence="1" />"#,

        r#"Recorded team rules and security guardrails.
<memory subject="User" property="Rule: Always enable two-factor authentication" type="RuleOrAlert" valence="-1" />
<memory subject="User" property="Rule: Never leave unlocked laptop unattended" type="RuleOrAlert" valence="-1" />"#,

        r#"Rescheduled upcoming security audit.
<memory subject="David" property="Security Audit: 10/10/2026" type="Object" valence="1" />
<memory subject="User" property="Audit Lead: David" type="Object" valence="1" />"#,

        r#"Update to audit date after team sync.
<forget subject="David" property="Security Audit: 10/10/2026" />
<memory subject="David" property="Security Audit: 24/10/2026" type="Object" valence="1" />"#,
    ];

    let mut total_learned = 0;
    for (i, turn) in turns.iter().enumerate() {
        let (_clean, learned) = engine.extract_and_apply_memory_tags(turn);
        assert!(!learned.is_empty(), "Turn {} should learn memories", i + 1);
        total_learned += learned.len();
    }

    assert!(total_learned >= 12, "Expected at least 12 learned memory tags");
    assert!(engine.find_node_by_label("Workstation: Triple Monitor Setup").is_some());
    assert!(engine.find_node_by_label("Security Audit: 24/10/2026").is_some());
    assert!(engine.find_node_by_label("Security Audit: 10/10/2026").is_none(), "Old audit date must be forgotten");

    let audit_ctx = engine.search_active_context_for_query("When is the security audit with David?");
    assert!(audit_ctx.is_some());
    assert!(audit_ctx.unwrap().contains("24/10/2026"));
}

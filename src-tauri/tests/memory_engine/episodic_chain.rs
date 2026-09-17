use app_lib::services::memory_engine::*;

#[test]
fn test_episodic_markdown_chain_linking() {
    let ep_dir = MemoryGraphEngine::episodios_dir();
    let _ = std::fs::create_dir_all(&ep_dir);

    let test_session_id = format!("session-test-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());

    // Turn 1
    let learned1 = vec!["Manuela -> Daughter of User".to_string()];
    let ep1_file = MemoryGraphEngine::record_episodic_turn(
        "Hello, I have a daughter named Manuela",
        "Pleased to meet you! I have recorded that your daughter is Manuela.",
        &learned1,
        Some(&test_session_id),
        Some("Family Chat"),
    ).expect("Should record turn 1");

    let ep1_path = ep_dir.join(&ep1_file);
    assert!(ep1_path.exists());
    let ep1_content = std::fs::read_to_string(&ep1_path).unwrap();
    assert!(ep1_content.contains("User Message") || ep1_content.contains("Mensagem do Usuário"));
    assert!(ep1_content.contains("Atena Response") || ep1_content.contains("Resposta da Atena"));
    assert!(ep1_content.contains("Manuela -> Daughter of User"));
    assert!(ep1_content.contains(&test_session_id));

    // Turn 2
    let learned2 = vec!["Manuela -> Age: 2 months".to_string()];
    let ep2_file = MemoryGraphEngine::record_episodic_turn(
        "Today she turned 2 months old",
        "Congratulations on Manuela's 2 months!",
        &learned2,
        Some(&test_session_id),
        Some("Family Chat"),
    ).expect("Should record turn 2");

    let ep2_path = ep_dir.join(&ep2_file);
    assert!(ep2_path.exists());
    let ep2_content = std::fs::read_to_string(&ep2_path).unwrap();

    // Episode 2 must link back to episode 1
    assert!(ep2_content.contains(&ep1_file));

    // Episode 1 must have been updated to point forward to episode 2
    let ep1_content_updated = std::fs::read_to_string(&ep1_path).unwrap();
    assert!(ep1_content_updated.contains(&ep2_file));

    // Test get_recent_episodes_context
    let ep_ctx = MemoryGraphEngine::get_recent_episodes_context(5);
    assert!(ep_ctx.is_some());
    let ep_ctx_str = ep_ctx.unwrap();
    assert!(ep_ctx_str.contains("[RECENT COGNITIVE EPISODE CHAIN]:") || ep_ctx_str.contains("[CADEIA DE EPISÓDIOS COGNITIVOS RECENTES]:"));
    assert!(ep_ctx_str.contains("Manuela"));

    // Test search_episodes
    let search_res = MemoryGraphEngine::search_episodes("Manuela", Some(&test_session_id), 5);
    assert_eq!(search_res.len(), 2);
    assert_eq!(search_res[0].session_id.as_deref(), Some(test_session_id.as_str()));

    // Test count by session
    let (ep_cnt, mem_cnt) = MemoryGraphEngine::count_session_episodes_and_memories(&test_session_id);
    assert_eq!(ep_cnt, 2);
    assert_eq!(mem_cnt, 2);
}

#[test]
fn test_session_scope_and_export() {
    let mut engine = MemoryGraphEngine::new(64);

    // Global fact
    engine.learn_from_text("Carlos lives in San Francisco");

    // Private fact tied to session
    let sid = "session-xyz-123";
    engine.learn_from_text_with_session("Carlos has a secret: Project Atena", Some(sid));

    let full = engine.get_full_graph();
    assert!(full.nodes.len() >= 2);

    let has_global = full.nodes.iter().any(|n| n.session_id.is_none());
    let has_private = full.nodes.iter().any(|n| n.session_id.as_deref() == Some(sid));
    assert!(has_global, "Should contain global nodes");
    assert!(has_private, "Should contain private nodes bound to session");

    // Test scope update
    let target_node_id = full.nodes.iter().find(|n| n.session_id.as_deref() == Some(sid)).unwrap().id;
    engine.update_node_scope(target_node_id, None).expect("Should update to global");
    assert!(engine.nodes.get(&target_node_id).unwrap().session_id.is_none());

    // Test serialization / deserialization v3
    let bytes = engine.serialize_to_bytes();
    let restored = MemoryGraphEngine::deserialize_from_bytes(&bytes, 64).expect("Should load v3");
    assert_eq!(restored.node_count(), engine.node_count());

    // Test Markdown export
    let md = engine.export_graph_markdown();
    assert!(md.contains("# Associative Memory Graph") || md.contains("# Grafo de Memória"));
    assert!(md.contains("Entities and Concepts") || md.contains("Entidades e Conceitos"));
    assert!(md.contains("Neural Connections") || md.contains("Conexões Neurais"));

    // Test JSON export
    let json_str = engine.export_graph_json().expect("Valid JSON export");
    assert!(json_str.contains("\"nodes\":"));
    assert!(json_str.contains("\"edges\":"));
}

#[test]
fn test_rebuild_from_episodes_if_empty() {
    let mut engine = MemoryGraphEngine::with_default_cache();
    engine.insert_declarative_fact("User", "Likes reading", 1, None, 1700000000);
    engine.insert_declarative_fact("Maria", "Friend of User", 1, None, 1700000001);
    engine.insert_rule_fact("Carlos", "Avoid noisy environments", None, 1700000002);
    engine.insert_container_fact("Keys", "Drawer", None, 1700000003);

    assert!(engine.find_node_by_label("User").is_some());
    assert!(engine.find_node_by_label("Maria").is_some());
    assert!(engine.find_node_by_label("Carlos").is_some());
    assert!(engine.find_node_by_label("Keys").is_some());
    assert!(engine.node_count() >= 4);
}

#[test]
fn test_load_default_or_init_restores_brain() {
    let engine = MemoryGraphEngine::load_default_or_init();
    assert!(engine.node_count() > 0);
}

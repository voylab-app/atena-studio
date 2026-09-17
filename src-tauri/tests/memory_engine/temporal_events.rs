use app_lib::core::memory::*;
use app_lib::services::memory_engine::*;

#[test]
fn test_smart_event_memory_and_temporal_resolution() {
    let mut engine = MemoryGraphEngine::with_default_cache();

    // Simulates AI response containing wedding tags and relative temporal resolution
    let ai_response = r#"Certainly! For your friend's wedding, I recommend a practical home gift.
<memory subject="User" property="Friend: Ana" type="Object" valence="1" />
<memory subject="Ana" property="Friend of User" type="Object" valence="1" />
<memory subject="Ana" property="Event: Wedding" type="Object" valence="1" />
<memory subject="Ana" property="Wedding Date: Tomorrow" type="RuleOrAlert" valence="1" />"#;

    let (_clean_text, learned) = engine.extract_and_apply_memory_tags(ai_response);
    assert_eq!(learned.len(), 4);

    // 1. Verify that 'Ana' and 'User' were created and connected
    let user_node = engine.find_node_by_label("User").expect("User must exist").clone();
    let ana_node = engine.find_node_by_label("Ana").expect("Ana must exist").clone();
    let edge_exists = engine
        .adjacency
        .get(&user_node.id)
        .map(|edges| edges.iter().any(|e| e.target_id == ana_node.id))
        .unwrap_or(false);
    assert!(edge_exists, "Edge from User to Ana must exist");

    // 2. Verify that 'Tomorrow' was auto-resolved to absolute date (DD/MM/YYYY)
    let tomorrow_str = (chrono::Local::now() + chrono::Duration::days(1)).format("%d/%m/%Y").to_string();
    let expected_date_prop = format!("Wedding Date: {}", tomorrow_str);
    assert!(
        engine.find_node_by_label(&expected_date_prop).is_some(),
        "Property must contain resolved absolute date: {}",
        expected_date_prop
    );

    // 3. Verify that event was not misclassified as negative RuleOrAlert
    let date_node = engine.find_node_by_label(&expected_date_prop).unwrap();
    assert_ne!(date_node.type_flag, NodeType::RuleOrAlert, "Wedding date must never be RuleOrAlert");
    assert_eq!(date_node.valence, 1, "Wedding date valence must be positive");
}

#[test]
fn test_memory_temporal_timestamps_and_relative_context() {
    let now = MemoryGraphEngine::current_timestamp();

    // 1. Test format_memory_timestamp in English
    let s_now = MemoryGraphEngine::format_memory_timestamp(now - 10, now);
    assert!(s_now.contains("just now"), "Expected 'just now', got: {}", s_now);

    let s_min = MemoryGraphEngine::format_memory_timestamp(now - 180, now);
    assert!(s_min.contains("3 min ago"), "Expected '3 min ago', got: {}", s_min);

    let s_hours = MemoryGraphEngine::format_memory_timestamp(now - 7200, now);
    assert!(s_hours.contains("today, 2 h ago"), "Expected 'today, 2 h ago', got: {}", s_hours);

    let s_yesterday = MemoryGraphEngine::format_memory_timestamp(now - 100_000, now);
    assert!(s_yesterday.contains("yesterday"), "Expected 'yesterday', got: {}", s_yesterday);

    let s_days = MemoryGraphEngine::format_memory_timestamp(now - 300_000, now);
    assert!(s_days.contains("3 days ago"), "Expected '3 days ago', got: {}", s_days);

    // 2. Test saving node and edge with timestamp via tags
    let mut engine = MemoryGraphEngine::with_default_cache();
    let ai_resp = r#"Saved successfully!
<memory subject="User" property="Bought a new book" type="Object" valence="1" />"#;
    let (_, learned) = engine.extract_and_apply_memory_tags(ai_resp);
    assert_eq!(learned.len(), 1);

    let user_created_at = engine.find_node_by_label("User").expect("User must exist").created_at;
    assert!(user_created_at > 0, "Node must have creation timestamp");

    let book_created_at = engine.find_node_by_label("Bought a new book").expect("Book node must exist").created_at;
    assert!(book_created_at > 0, "Book node must have creation timestamp");

    // 3. Test get_recent_memory_records contains the English tag [saved at: ...]
    let recent = engine.get_recent_memory_records(5).expect("Should return recent records");
    println!("Formatted recent records:\n{}", recent);
    assert!(recent.contains("[saved at:"), "Should contain '[saved at:' tag");
    assert!(recent.contains("Bought a new book"));

    // 4. Test build_llm_context with timestamp
    let ctx = engine.get_user_profile_context().expect("Should return user profile");
    println!("Associative context with timestamp:\n{}", ctx);
    assert!(ctx.contains("[saved at:"), "Active context must contain '[saved at:'");

    // 5. Test serialization and deserialization V2 (with created_at)
    let bytes_v2 = engine.serialize_to_bytes();
    let restored_v2 = MemoryGraphEngine::deserialize_from_bytes(&bytes_v2, 64)
        .expect("Should deserialize V2");
    assert_eq!(restored_v2.node_count(), engine.node_count());
    assert_eq!(restored_v2.edge_count(), engine.edge_count());

    let restored_book = restored_v2.find_node_by_label("Bought a new book").unwrap();
    assert_eq!(restored_book.created_at, book_created_at);
}

#[test]
fn test_relative_temporal_resolution_comprehensive() {
    let now = chrono::Local::now();
    let tomorrow_str = (now + chrono::Duration::days(1)).format("%d/%m/%Y").to_string();
    let yesterday_str = (now - chrono::Duration::days(1)).format("%d/%m/%Y").to_string();
    let today_str = now.format("%d/%m/%Y").to_string();

    assert_eq!(
        MemoryGraphEngine::resolve_relative_temporal_date("Project Deadline: Tomorrow"),
        format!("Project Deadline: {}", tomorrow_str)
    );
    assert_eq!(
        MemoryGraphEngine::resolve_relative_temporal_date("Team Sync: Today"),
        format!("Team Sync: {}", today_str)
    );
    assert_eq!(
        MemoryGraphEngine::resolve_relative_temporal_date("Code Freeze: Yesterday"),
        format!("Code Freeze: {}", yesterday_str)
    );
    assert_eq!(
        MemoryGraphEngine::resolve_relative_temporal_date("Data da Entrega: Amanhã"),
        format!("Data da Entrega: {}", tomorrow_str)
    );
    assert_eq!(
        MemoryGraphEngine::resolve_relative_temporal_date("Reunião: Ontem"),
        format!("Reunião: {}", yesterday_str)
    );
    assert_eq!(
        MemoryGraphEngine::resolve_relative_temporal_date("Evento: Hoje"),
        format!("Evento: {}", today_str)
    );
}

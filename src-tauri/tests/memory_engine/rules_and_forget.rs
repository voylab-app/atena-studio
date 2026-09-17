use app_lib::services::memory_engine::*;

#[test]
fn test_forget_and_semantic_date_conflict_resolution() {
    let mut engine = MemoryGraphEngine::with_default_cache();

    // 1. Initial state with initial date and relationship
    let ai_turn1 = r#"Recording wedding details.
<memory subject="User" property="Friend of Maria" type="Object" valence="1" />
<memory subject="Maria" property="Wedding: 01/09/2026" type="Object" valence="1" />"#;
    engine.extract_and_apply_memory_tags(ai_turn1);

    let user_id = engine.find_node_by_label("User").unwrap().id;
    let maria_id = engine.find_node_by_label("Maria").unwrap().id;

    assert!(engine.adjacency.get(&user_id).unwrap().iter().any(|e| e.target_id == maria_id));
    assert!(engine.adjacency.get(&maria_id).unwrap().iter().any(|e| e.target_id == user_id));

    // 2. Test get_recent_memory_records
    let recent = engine.get_recent_memory_records(10).expect("Should return recent records");
    assert!(recent.contains("Maria ->"));

    // 3. AI emits forget for obsolete date and registers new date & gift
    let ai_turn2 = r#"Correcting obsolete data with new rescheduled date and gift.
<forget subject="Maria" property="Wedding: 01/09/2026" />
<memory subject="Maria" property="Wedding Date: 14/11/2026" type="Object" valence="1" />
<memory subject="Maria" property="Wedding Gift: Air Fryer" type="Object" valence="1" />"#;
    let (_clean, learned) = engine.extract_and_apply_memory_tags(ai_turn2);
    assert_eq!(learned.len(), 3);

    let maria_edges = engine.adjacency.get(&maria_id).unwrap();
    let new_date = engine.find_node_by_label("Wedding Date: 14/11/2026").unwrap().id;
    let gift = engine.find_node_by_label("Wedding Gift: Air Fryer").unwrap().id;

    assert!(maria_edges.iter().any(|e| e.target_id == new_date));
    assert!(maria_edges.iter().any(|e| e.target_id == gift));
    assert!(engine.find_node_by_label("Wedding: 01/09/2026").is_none());
}

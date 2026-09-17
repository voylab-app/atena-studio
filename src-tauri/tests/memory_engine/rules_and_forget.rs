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

#[test]
fn test_cross_lingual_search_and_leaf_activation() {
    let mut engine = MemoryGraphEngine::with_default_cache();

    let ai_turn = r#"Adding allergy rule and coffee grinder.
<memory subject="User" property="Rule: Allergy: Seafood (frutos do mar)" type="RuleOrAlert" valence="-1" />
<memory subject="User" property="Coffee Grinder: Comandante C40 MK4 Nitro Blade manual grinder" type="Object" valence="1" />"#;
    engine.extract_and_apply_memory_tags(ai_turn);

    // 1. Spreading activation on "Allergy"
    let paths = engine.traverse_associations("Allergy", 3).expect("Should find path");
    assert!(!paths.is_empty(), "Paths should not be empty for Allergy query");
    let llm_context = MemoryGraphEngine::build_llm_context(&paths);
    assert!(!llm_context.contains("no association found"));
    assert!(llm_context.contains("Allergy: Seafood"));

    // 2. Cross-lingual search for "alergia" in Portuguese
    let allergy_res = engine.search_active_context_for_query("alergia");
    assert!(allergy_res.is_some(), "Search for 'alergia' should find 'Allergy: Seafood'");
    assert!(allergy_res.unwrap().contains("Allergy"));

    // 3. Cross-lingual search for "moedor" in Portuguese
    let grinder_res = engine.search_active_context_for_query("moedor");
    assert!(grinder_res.is_some(), "Search for 'moedor' should find 'Coffee Grinder'");
    assert!(grinder_res.unwrap().contains("Comandante"));

    // 4. Accent-tolerant search for "café" in Portuguese
    let cafe_res = engine.search_active_context_for_query("café");
    assert!(cafe_res.is_some(), "Search for 'café' with accent should find Coffee Grinder");
    assert!(cafe_res.unwrap().contains("Comandante"));
}


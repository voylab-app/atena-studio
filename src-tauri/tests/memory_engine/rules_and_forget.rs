use app_lib::core::memory::*;
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

#[test]
fn test_shared_rules_across_multiple_entities() {
    let mut engine = MemoryGraphEngine::with_default_cache();

    let tag1 = r#"<memory subject="User" property="Rule: Allergy: Milk" type="RuleOrAlert" valence="-1" />"#;
    let tag2 = r#"<memory subject="Carlos" property="Rule: Allergy: Milk" type="RuleOrAlert" valence="-1" />"#;
    let tag3 = r#"<memory subject="Ana" property="Allergy: Milk" type="RuleOrAlert" valence="-1" />"#;
    engine.extract_and_apply_memory_tags(tag1);
    engine.extract_and_apply_memory_tags(tag2);
    engine.extract_and_apply_memory_tags(tag3);

    // Verify there is only ONE rule node for Allergy: Milk
    let rule_nodes: Vec<_> = engine
        .nodes
        .values()
        .filter(|n| n.label.to_lowercase().contains("allergy: milk"))
        .collect();
    assert_eq!(rule_nodes.len(), 1, "Expected exactly 1 consolidated rule node, found {}", rule_nodes.len());

    let rule_id = rule_nodes[0].id;
    let user_id = engine.find_node_by_label("User").unwrap().id;
    let carlos_id = engine.find_node_by_label("Carlos").unwrap().id;
    let ana_id = engine.find_node_by_label("Ana").unwrap().id;

    // User, Carlos, and Ana must all have AvoidAction edges pointing to the same rule node
    assert!(engine.adjacency.get(&user_id).unwrap().iter().any(|e| e.target_id == rule_id));
    assert!(engine.adjacency.get(&carlos_id).unwrap().iter().any(|e| e.target_id == rule_id));
    assert!(engine.adjacency.get(&ana_id).unwrap().iter().any(|e| e.target_id == rule_id));

    // When Carlos and Ana are deleted, the rule must NOT be deleted because User still uses it
    let _ = engine.delete_node(carlos_id);
    let _ = engine.delete_node(ana_id);
    assert!(engine.nodes.contains_key(&rule_id));

    // When User is deleted, the rule node has 0 connections and must be pruned
    let _ = engine.delete_node(user_id);
    assert!(!engine.nodes.contains_key(&rule_id));
}

#[test]
fn test_scrub_heals_duplicate_and_orphan_rules() {
    let mut engine = MemoryGraphEngine::with_default_cache();

    // 1. Manually simulate a corrupted state with duplicate nodes and an orphan
    let user_id = engine.add_node(NodeType::Object, "User");
    let canonical_rule_id = engine.add_node_with_valence(NodeType::RuleOrAlert, "Rule: Allergy: Peanuts", -1);
    let orphan_rule_id = engine.add_node_with_valence(NodeType::RuleOrAlert, "Rule: Allergy: Peanuts", -1);
    let _ = engine.add_edge(user_id, canonical_rule_id, RelationType::AvoidAction);

    assert_eq!(engine.node_count(), 3);
    assert!(!engine.has_any_connections(orphan_rule_id));

    // 2. Run scrub_corrupted_nodes
    engine.scrub_corrupted_nodes();

    // 3. Should now have only 2 nodes (User and 1 single rule node)
    assert_eq!(engine.node_count(), 2);
    let remaining_rule = engine.find_node_by_label("Rule: Allergy: Peanuts").expect("Rule should exist");
    assert!(engine.has_any_connections(remaining_rule.id));
    assert!(!engine.nodes.contains_key(&orphan_rule_id));
}

#[test]
fn test_bidirectional_and_prefix_tolerant_forget() {
    let mut engine = MemoryGraphEngine::with_default_cache();

    let tag = r#"<memory subject="User" property="Rule: Allergy: Gluten" type="RuleOrAlert" valence="-1" />"#;
    engine.extract_and_apply_memory_tags(tag);

    assert!(engine.find_node_by_label("Rule: Allergy: Gluten").is_some());

    // Forget without "Rule:" prefix
    let forget_tag = r#"<forget subject="User" property="Allergy: Gluten" />"#;
    let (_clean, learned) = engine.extract_and_apply_memory_tags(forget_tag);
    assert!(!learned.is_empty());

    // The orphan rule should have been pruned automatically
    assert!(engine.find_node_by_label("Rule: Allergy: Gluten").is_none());
}

#[test]
fn test_query_entity_allergy_retrieval() {
    let mut engine = MemoryGraphEngine::with_default_cache();

    // User facts
    let user_tag = r#"
    <memory subject="User" property="Name: Maria Silva" type="Object" valence="1" />
    <memory subject="User" property="Age: 30 years old" type="Object" valence="1" />
    <memory subject="User" property="Hobby: Likes fishing" type="Object" valence="1" />
    <memory subject="User" property="Plan: Fishing trip with Carlos on 20/09/2026" type="Object" valence="1" />
    <memory subject="User" property="Rule: Allergy: Milk (avoid dairy products)" type="RuleOrAlert" valence="-1" />
    "#;
    engine.extract_and_apply_memory_tags(user_tag);

    // Friend facts
    let friend_tag = r#"
    <memory subject="Carlos" property="Friend of Maria" type="Object" valence="1" />
    <memory subject="Carlos" property="Hobby: Likes fishing" type="Object" valence="1" />
    <memory subject="Carlos" property="Rule: Allergy: Milk (avoid dairy products)" type="RuleOrAlert" valence="-1" />
    "#;
    engine.extract_and_apply_memory_tags(friend_tag);

    let ctx = engine.search_active_context_for_query("can Carlos eat?");
    assert!(ctx.is_some());
    let c = ctx.unwrap();
    assert!(c.contains("Allergy: Milk"), "Context should contain Allergy: Milk for Carlos, got: {}", c);
}

#[test]
fn test_ignore_memory_tags_in_code_blocks() {
    let mut engine = MemoryGraphEngine::with_default_cache();

    // 1. Initial valid memory
    let initial_tag = r#"<memory subject="Carlos" property="Rule: Allergy: Milk" type="RuleOrAlert" valence="-1" />"#;
    engine.extract_and_apply_memory_tags(initial_tag);
    assert!(engine.find_node_by_label("Carlos").is_some());
    assert!(engine.find_node_by_label("Rule: Allergy: Milk").is_some());

    // 2. AI produces response with example in backticks and code blocks
    let response_with_snippets = r#"
Here is an explanation of what happened:
Possible cause: `<forget subject="Carlos" property="*Allergy*" />`
And in json:
```json
<memory subject="Carlos" property="Fake Fact" type="Object" valence="1" />
```
Real tag outside code:
<memory subject="Carlos" property="Real Fact: Painter" type="Object" valence="1" />
"#;

    let (clean_text, learned) = engine.extract_and_apply_memory_tags(response_with_snippets);

    // Should NOT forget Carlos allergy
    assert!(engine.find_node_by_label("Rule: Allergy: Milk").is_some(), "Allergy rule should NOT be forgotten by code block snippet");
    // Should NOT learn Fake Fact
    assert!(engine.find_node_by_label("Fake Fact").is_none(), "Fake fact in code block should NOT be learned");
    // Should learn Real Fact
    assert!(engine.find_node_by_label("Real Fact: Painter").is_some(), "Real fact outside code block SHOULD be learned");
    // The snippet in backticks should still be in clean_text
    assert!(clean_text.contains("<forget subject=\"Carlos\""), "Clean text must preserve code snippet inside backticks");
    assert_eq!(learned.len(), 1);
}

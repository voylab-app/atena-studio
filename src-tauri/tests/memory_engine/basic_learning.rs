use app_lib::services::memory_engine::*;

#[test]
fn test_real_dialogue_learning_and_query() {
    let mut engine = MemoryGraphEngine::with_default_cache();

    // 1. Message 1 (Portuguese dialogue)
    let text1 = "Oi atena eu me chamo Maria, tenho olhos castanhos, cabelo preto, e sou arquiteta";
    let l1 = engine.learn_from_text(text1);
    assert!(!l1.is_empty());
    assert!(engine.find_node_by_label("Maria").is_some());

    // 2. Message 2
    let text2 = "Carlos e meu marido, ele tem cabelos castanhos, o olho dele e da cor do meu, ele ta usando aparelho nos dentes, temos uma filha camada Luna";
    let l2 = engine.learn_from_text(text2);
    assert!(!l2.is_empty());
    assert!(engine.find_node_by_label("Carlos").is_some());
    assert!(engine.find_node_by_label("Luna").is_some());

    // 3. Test phrases without accents
    let text3 = "Maria e formada";
    let l3 = engine.learn_from_text(text3);
    assert!(!l3.is_empty(), "Should learn 'Maria e formada'");
    assert!(engine.find_node_by_label("Formada").is_some() || engine.find_node_by_label("formada").is_some());

    // 4. Test spreading activation with query
    let query_res = engine.traverse_associations("quem e a filha de Maria", 3);
    assert!(query_res.is_ok(), "Query failed: {:?}", query_res);
    let paths = query_res.unwrap();
    assert!(!paths.is_empty());

    let context = MemoryGraphEngine::build_llm_context(&paths);
    println!("Generated LLM Context:\n{}", context);
    assert!(context.contains("Maria"));
}

#[test]
fn test_english_dialogue_learning_and_query() {
    let mut engine = MemoryGraphEngine::with_default_cache();
    let ai_resp = r#"I have noted your friend's upcoming wedding!
<memory subject="User" property="Friend: Maria" type="Object" valence="1" />
<memory subject="Maria" property="Friend of User" type="Object" valence="1" />
<memory subject="Maria" property="Event: Wedding" type="Object" valence="1" />
<memory subject="Maria" property="Wedding Date: Tomorrow" type="Object" valence="1" />"#;

    let (_clean, learned) = engine.extract_and_apply_memory_tags(ai_resp);
    assert_eq!(learned.len(), 4);
    assert!(engine.find_node_by_label("Maria").is_some());

    // Test active context search with English stop words ("When is", "'s")
    let ctx = engine.search_active_context_for_query("When is Maria's wedding?");
    assert!(ctx.is_some(), "Should find active context for Maria");
    let c_str = ctx.unwrap();
    assert!(c_str.contains("Maria"));
    assert!(c_str.contains("HAS_PROPERTY"));
}

#[test]
fn test_autonomous_memory_updates_and_arbitrary_facts() {
    let mut engine = MemoryGraphEngine::with_default_cache();

    // Turn 1: AI saves friend and initial date
    let ai_turn1 = r#"I suggest a thoughtful gift for your friend Maria's wedding.
<memory subject="User" property="Friend: Maria" type="Object" valence="1" />
<memory subject="Maria" property="Friend of User" type="Object" valence="1" />
<memory subject="Maria" property="Wedding Date: 01/09/2026" type="Object" valence="1" />"#;
    engine.extract_and_apply_memory_tags(ai_turn1);

    let maria_id = engine.find_node_by_label("Maria").unwrap().id;
    let old_date_id = engine.find_node_by_label("Wedding Date: 01/09/2026").unwrap().id;
    assert!(engine.adjacency.get(&maria_id).unwrap().iter().any(|e| e.target_id == old_date_id));

    // Turn 2: AI autonomously updates the rescheduled date and bought gift
    let ai_turn2 = r#"Got it! The wedding date was rescheduled and I recorded the bought gift.
<memory subject="Maria" property="Wedding Date: 14/11/2026" type="Object" valence="1" />
<memory subject="Maria" property="Wedding Gift: Air Fryer" type="Object" valence="1" />
<memory subject="User" property="Bought gift for Maria: Air Fryer" type="Object" valence="1" />"#;
    let (_clean, learned2) = engine.extract_and_apply_memory_tags(ai_turn2);
    assert_eq!(learned2.len(), 3);

    // Old date 01/09/2026 should be superseded by 14/11/2026 on Maria's adjacency
    let new_date_id = engine.find_node_by_label("Wedding Date: 14/11/2026").unwrap().id;
    let gift_id = engine.find_node_by_label("Wedding Gift: Air Fryer").unwrap().id;
    let maria_edges = engine.adjacency.get(&maria_id).unwrap();

    assert!(maria_edges.iter().any(|e| e.target_id == new_date_id), "Should have new date");
    assert!(!maria_edges.iter().any(|e| e.target_id == old_date_id), "Should no longer have old date");
    assert!(maria_edges.iter().any(|e| e.target_id == gift_id), "Should have bought gift");
}

#[test]
fn test_inspect_user_brain() {
    let root = MemoryGraphEngine::brain_root_path();
    let files = [
        root.join("brain_manifest.atena"),
        root.join("temporal").join("identidades.atena"),
        root.join("parietal").join("associacoes_locais.atena"),
        root.join("prefrontal").join("regras_preferencias.atena"),
        root.join("occipital").join("formas_visuals.atena"),
    ];
    for f in &files {
        if f.exists() {
            println!("--- FILE: {} ---", f.display());
            if let Ok(eng) = MemoryGraphEngine::load_from_compressed_binary(f, 64) {
                for (id, node) in &eng.nodes {
                    println!("  Node {}: {:?} (valence: {})", id, node.label, node.valence);
                }
                for (src, edges) in &eng.adjacency {
                    for e in edges {
                        let src_lbl = eng.nodes.get(src).map(|n| n.label.as_str()).unwrap_or("?");
                        let tgt_lbl = eng.nodes.get(&e.target_id).map(|n| n.label.as_str()).unwrap_or("?");
                        println!("    {} -> {:?} -> {}", src_lbl, e.relation_type, tgt_lbl);
                    }
                }
            }
        }
    }
}

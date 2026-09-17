// =============================================================================
// bin/memory_demo.rs — Demonstração do Motor de Memória Associativa
// =============================================================================
//
// Execute com: cargo run --bin memory_demo
//
// Demonstra:
//   1. Criação de nós e arestas
//   2. Salvamento comprimido (.atena) em Zstd e LZ4
//   3. Leitura de volta para a RAM com validação
//   4. Busca associativa via Spreading Activation
//   5. Reforço sináptico (aprendizado)
//   6. Decaimento sináptico (esquecimento)
//   7. Hot Cache hit/miss statistics
//   8. Saída do sintetizador LLM



// Importar os módulos do crate principal
use app_lib::core::memory::*;
use app_lib::services::memory_engine::MemoryGraphEngine;

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║   ATENA STUDIO — Motor de Memória Associativa Procedural   ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    // =========================================================================
    // 1. Criação dos Nós
    // =========================================================================
    println!("━━━ 1. CRIAÇÃO DE NÓS ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let mut engine = MemoryGraphEngine::with_default_cache();

    let lapis_id = engine.add_node(NodeType::Object, "Lápis Azul");
    let camisa_id = engine.add_node(NodeType::Object, "Camisa Rosa");
    let caixa_id = engine.add_node(NodeType::Container, "Caixa 2");
    let marca_id = engine.add_node(NodeType::Attribute, "Marca Faber");

    println!("  [+] Nó id={}: Lápis Azul (Object)", lapis_id);
    println!("  [+] Nó id={}: Camisa Rosa (Object)", camisa_id);
    println!("  [+] Nó id={}: Caixa 2 (Container)", caixa_id);
    println!("  [+] Nó id={}: Marca Faber (Attribute)", marca_id);
    println!("  Total de nós: {}", engine.node_count());
    println!();

    // =========================================================================
    // 2. Criação das Arestas (Conexões)
    // =========================================================================
    println!("━━━ 2. CRIAÇÃO DE ARESTAS ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Lápis Azul -> GUARDADO_COM -> Camisa Rosa
    engine
        .add_edge(lapis_id, camisa_id, RelationType::StoredWith)
        .expect("Falha ao adicionar aresta");
    println!(
        "  [→] Lápis Azul --(GUARDADO_COM)--> Camisa Rosa"
    );

    // Lápis Azul -> LOCALIZADO_EM -> Caixa 2
    engine
        .add_edge(lapis_id, caixa_id, RelationType::LocatedIn)
        .expect("Falha ao adicionar aresta");
    println!(
        "  [→] Lápis Azul --(LOCALIZADO_EM)--> Caixa 2"
    );

    // Camisa Rosa -> LOCALIZADO_EM -> Caixa 2
    engine
        .add_edge(camisa_id, caixa_id, RelationType::LocatedIn)
        .expect("Falha ao adicionar aresta");
    println!(
        "  [→] Camisa Rosa --(LOCALIZADO_EM)--> Caixa 2"
    );

    // Lápis Azul -> TEM_PROPRIEDADE -> Marca Faber
    engine
        .add_edge(lapis_id, marca_id, RelationType::HasProperty)
        .expect("Falha ao adicionar aresta");
    println!(
        "  [→] Lápis Azul --(TEM_PROPRIEDADE)--> Marca Faber"
    );

    println!("  Total de arestas: {}", engine.edge_count());
    println!();

    // =========================================================================
    // 3. Serialização Binária (sem compressão) — Verificação de Integridade
    // =========================================================================
    println!("━━━ 3. SERIALIZAÇÃO BINÁRIA ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let raw_bytes = engine.serialize_to_bytes();
    println!("  Buffer serializado: {} bytes", raw_bytes.len());

    // Verificar que deserialização reconstrói o grafo fielmente
    let restored =
        MemoryGraphEngine::deserialize_from_bytes(&raw_bytes, 128).expect("Falha na deserialização");
    println!(
        "  Grafo restaurado: {} nós, {} arestas ✓",
        restored.node_count(),
        restored.edge_count()
    );
    assert_eq!(engine.node_count(), restored.node_count());
    assert_eq!(engine.edge_count(), restored.edge_count());
    println!();

    // =========================================================================
    // 4. Salvamento Comprimido em Disco
    // =========================================================================
    println!("━━━ 4. SALVAMENTO COMPRIMIDO ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let tmp_dir = std::env::temp_dir().join("atena_memory_demo");
    std::fs::create_dir_all(&tmp_dir).expect("Falha ao criar diretório temporário");

    // --- Zstd ---
    let zstd_path = tmp_dir.join("memory_demo.zstd.atena");
    let zstd_size = engine
        .save_to_compressed_binary(&zstd_path, CompressionType::Zstd)
        .expect("Falha ao salvar Zstd");
    let ratio_zstd = (zstd_size as f64 / raw_bytes.len() as f64) * 100.0;
    println!(
        "  [ZSTD] Salvo: {} -> {} bytes ({:.1}% do original)",
        raw_bytes.len(),
        zstd_size,
        ratio_zstd
    );
    println!("         Arquivo: {}", zstd_path.display());

    // --- LZ4 ---
    let lz4_path = tmp_dir.join("memory_demo.lz4.atena");
    let lz4_size = engine
        .save_to_compressed_binary(&lz4_path, CompressionType::Lz4)
        .expect("Falha ao salvar LZ4");
    let ratio_lz4 = (lz4_size as f64 / raw_bytes.len() as f64) * 100.0;
    println!(
        "  [LZ4]  Salvo: {} -> {} bytes ({:.1}% do original)",
        raw_bytes.len(),
        lz4_size,
        ratio_lz4
    );
    println!("         Arquivo: {}", lz4_path.display());

    // --- Uncompressed (referência) ---
    let raw_path = tmp_dir.join("memory_demo.raw.atena");
    let raw_size = engine
        .save_to_compressed_binary(&raw_path, CompressionType::Uncompressed)
        .expect("Falha ao salvar raw");
    println!(
        "  [RAW]  Salvo: {} bytes (sem compressão, referência)",
        raw_size
    );
    println!();

    // =========================================================================
    // 5. Leitura de Volta para a RAM
    // =========================================================================
    println!("━━━ 5. LEITURA COMPRIMIDA → RAM ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let mut loaded_zstd =
        MemoryGraphEngine::load_from_compressed_binary(&zstd_path, 128)
            .expect("Falha ao carregar Zstd");
    println!(
        "  [ZSTD] Carregado: {} nós, {} arestas ✓",
        loaded_zstd.node_count(),
        loaded_zstd.edge_count()
    );

    let loaded_lz4 = MemoryGraphEngine::load_from_compressed_binary(&lz4_path, 128)
        .expect("Falha ao carregar LZ4");
    println!(
        "  [LZ4]  Carregado: {} nós, {} arestas ✓",
        loaded_lz4.node_count(),
        loaded_lz4.edge_count()
    );

    // Verificar integridade do nó buscado
    let found = loaded_zstd
        .find_node_by_label("Lápis Azul")
        .expect("Nó 'Lápis Azul' não encontrado após reload");
    println!(
        "  Verificação: Nó '{}' id={} (type={:?}) ✓",
        found.label, found.id, found.type_flag
    );
    println!();

    // =========================================================================
    // 6. Busca Associativa — Spreading Activation
    // =========================================================================
    println!("━━━ 6. SPREADING ACTIVATION ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let paths = loaded_zstd
        .traverse_associations("Lápis Azul", 3)
        .expect("Falha na varredura associativa");

    println!(
        "  Busca a partir de \"Lápis Azul\" (max_depth=3):"
    );
    for (i, path) in paths.iter().enumerate() {
        let labels: Vec<String> = path
            .steps
            .iter()
            .map(|s| {
                if let Some(rel) = &s.relation {
                    format!("--{}-->  {}", rel.label(), s.node_label)
                } else {
                    s.node_label.clone()
                }
            })
            .collect();
        println!(
            "    Caminho {}: {} (peso={:.2})",
            i + 1,
            labels.join("  "),
            path.total_weight
        );
    }
    println!();

    // =========================================================================
    // 7. Sintetizador LLM
    // =========================================================================
    println!("━━━ 7. SINTETIZADOR LLM ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let llm_context = MemoryGraphEngine::build_llm_context(&paths);
    println!("  {}", llm_context);
    println!();

    // =========================================================================
    // 8. Reforço Sináptico — Aprendizado por Repetição
    // =========================================================================
    println!("━━━ 8. PLASTICIDADE SINÁPTICA — REFORÇO ━━━━━━━━━━━━━━━━━━━━━");

    println!("  Simulando 5 acessos repetidos: Lápis Azul -> Camisa Rosa");
    for i in 1..=5 {
        loaded_zstd
            .reinforce_edge(lapis_id, camisa_id)
            .expect("Falha no reforço");
        println!("    Reforço #{}: peso da aresta incrementado", i);
    }

    // Buscar novamente para ver o efeito do reforço
    let paths_after = loaded_zstd
        .traverse_associations("Lápis Azul", 3)
        .expect("Falha na varredura pós-reforço");

    let llm_after = MemoryGraphEngine::build_llm_context(&paths_after);
    println!("  Contexto LLM pós-reforço:");
    println!("  {}", llm_after);
    println!();

    // =========================================================================
    // 9. Decaimento Sináptico — Esquecimento Natural
    // =========================================================================
    println!("━━━ 9. PLASTICIDADE SINÁPTICA — DECAIMENTO ━━━━━━━━━━━━━━━━━━");

    println!("  Arestas antes do decay: {}", loaded_zstd.edge_count());
    println!("  Aplicando decay_factor=0.05 (5% de perda por ciclo)...");
    loaded_zstd.apply_decay(0.05);
    println!("  Arestas após o decay: {}", loaded_zstd.edge_count());

    // Decay agressivo para demonstrar esquecimento
    println!("  Aplicando 50 ciclos de decay agressivo (decay=0.15)...");
    for _ in 0..50 {
        loaded_zstd.apply_decay(0.15);
    }
    println!(
        "  Arestas sobreviventes: {} (arestas fracas foram esquecidas)",
        loaded_zstd.edge_count()
    );
    println!();

    // =========================================================================
    // 10. Estatísticas do Hot Cache
    // =========================================================================
    println!("━━━ 10. ESTATÍSTICAS DO MOTOR ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let stats = loaded_zstd.stats();
    println!("  Nós:           {}", stats.total_nodes);
    println!("  Arestas:       {}", stats.total_edges);
    println!("  Cache size:    {}", stats.cache_size);
    println!("  Cache hits:    {}", stats.cache_hits);
    println!("  Cache misses:  {}", stats.cache_misses);
    println!("  Hit rate:      {:.1}%", stats.cache_hit_rate);
    println!();

    // =========================================================================
    // Limpeza
    // =========================================================================
    println!("━━━ LIMPEZA ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let _ = std::fs::remove_dir_all(&tmp_dir);
    println!("  Arquivos temporários removidos: {}", tmp_dir.display());
    println!();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                 DEMO CONCLUÍDA COM SUCESSO ✓               ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}

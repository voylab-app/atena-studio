use app_lib::core::memory::*;
use app_lib::services::memory_engine::*;

#[test]
fn test_v1_backwards_compatibility() {
    let mut buf_v1 = Vec::new();
    // node_count = 2
    buf_v1.extend_from_slice(&2u32.to_le_bytes());

    // Node 1: User
    buf_v1.extend_from_slice(&1u32.to_le_bytes());
    buf_v1.push(NodeType::Object.as_byte());
    buf_v1.push(0u8);
    buf_v1.extend_from_slice(&4u16.to_le_bytes());
    buf_v1.extend_from_slice(b"User");

    // Node 2: Cafe
    buf_v1.extend_from_slice(&2u32.to_le_bytes());
    buf_v1.push(NodeType::Attribute.as_byte());
    buf_v1.push(1u8);
    buf_v1.extend_from_slice(&4u16.to_le_bytes());
    buf_v1.extend_from_slice(b"Cafe");

    // edge_count = 1
    buf_v1.extend_from_slice(&1u32.to_le_bytes());

    // Edge 1 (25 bytes)
    buf_v1.extend_from_slice(&1u32.to_le_bytes()); // src
    buf_v1.extend_from_slice(&2u32.to_le_bytes()); // tgt
    buf_v1.push(RelationType::HasProperty.as_byte()); // rel
    buf_v1.extend_from_slice(&1.0f32.to_le_bytes()); // weight
    buf_v1.extend_from_slice(&5u32.to_le_bytes()); // access_count
    buf_v1.extend_from_slice(&1700000000u64.to_le_bytes()); // last_accessed

    assert_eq!(buf_v1.len(), 4 + 12 + 12 + 4 + 25);

    let engine_v1 = MemoryGraphEngine::deserialize_from_bytes_version(&buf_v1, 64, 1)
        .expect("Should load legacy v1 buffer");

    assert_eq!(engine_v1.node_count(), 2);
    assert_eq!(engine_v1.edge_count(), 1);

    let user_edges = engine_v1.adjacency.get(&1).unwrap();
    assert_eq!(user_edges.len(), 1);
    assert_eq!(user_edges[0].last_accessed, 1700000000);
    assert_eq!(user_edges[0].created_at, 1700000000);

    // Re-serialize in v2 format
    let buf_v2 = engine_v1.serialize_to_bytes();
    let engine_v2 = MemoryGraphEngine::deserialize_from_bytes(&buf_v2, 64)
        .expect("Should load updated v2 buffer");
    assert_eq!(engine_v2.node_count(), 2);
    assert_eq!(engine_v2.edge_count(), 1);
}

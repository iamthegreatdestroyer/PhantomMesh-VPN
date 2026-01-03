//! Criterion Benchmarks for Agent Framework
//! Micro-benchmarks for individual components

#[cfg(test)]
mod benchmarks {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    use phantom_mesh::agent_framework::message::{AgentId, Message, MessageType, Priority};

    fn benchmark_message_creation(c: &mut Criterion) {
        c.bench_function("create_simple_message", |b| {
            b.iter(|| {
                Message::new(
                    black_box(AgentId::new("apex")),
                    black_box(vec![AgentId::new("fortress")]),
                    black_box(MessageType::Command("test".to_string())),
                    black_box(Priority::Normal),
                )
            })
        });
    }

    fn benchmark_message_with_payload(c: &mut Criterion) {
        c.bench_function("create_message_with_payload", |b| {
            b.iter(|| {
                Message::new(
                    black_box(AgentId::new("apex")),
                    black_box(vec![AgentId::new("fortress")]),
                    black_box(MessageType::Command("test".to_string())),
                    black_box(Priority::High),
                )
                .with_data("key1", serde_json::json!("value1"))
                .with_data("key2", serde_json::json!(42))
            })
        });
    }

    fn benchmark_message_builder(c: &mut Criterion) {
        c.bench_function("message_builder_construction", |b| {
            b.iter(|| {
                phantom_mesh::agent_framework::message::MessageBuilder::new(
                    black_box(AgentId::new("apex")),
                    black_box(MessageType::Command("test".to_string())),
                )
                .to(black_box(AgentId::new("fortress")))
                .priority(black_box(Priority::High))
                .data("data", serde_json::json!({"nested": "value"}))
                .build()
            })
        });
    }

    criterion_group!(
        benches,
        benchmark_message_creation,
        benchmark_message_with_payload,
        benchmark_message_builder,
    );
    criterion_main!(benches);
}

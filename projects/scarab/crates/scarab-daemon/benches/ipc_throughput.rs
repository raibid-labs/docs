//! IPC throughput and latency benchmarks

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::sync::mpsc as tokio_mpsc;

#[derive(Clone)]
struct Message {
    id: u64,
    payload: Vec<u8>,
    timestamp: Instant,
}

impl Message {
    fn new(id: u64, size: usize) -> Self {
        Self {
            id,
            payload: vec![0xAB; size],
            timestamp: Instant::now(),
        }
    }

    fn latency(&self) -> Duration {
        self.timestamp.elapsed()
    }
}

fn bench_mpsc_channel(c: &mut Criterion) {
    let mut group = c.benchmark_group("ipc_mpsc");

    for msg_size in [64, 256, 1024, 4096, 16384].iter() {
        group.throughput(Throughput::Bytes(*msg_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(msg_size),
            msg_size,
            |b, &msg_size| {
                b.iter(|| {
                    let (tx, rx) = mpsc::channel();
                    let messages = 1000;

                    let handle = thread::spawn(move || {
                        for _ in 0..messages {
                            let msg = rx.recv().unwrap();
                            black_box(msg);
                        }
                    });

                    for i in 0..messages {
                        tx.send(Message::new(i, msg_size)).unwrap();
                    }

                    handle.join().unwrap();
                });
            },
        );
    }

    group.finish();
}

fn bench_crossbeam_channel(c: &mut Criterion) {
    let mut group = c.benchmark_group("ipc_crossbeam");

    for msg_size in [64, 256, 1024, 4096, 16384].iter() {
        group.throughput(Throughput::Bytes(*msg_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(msg_size),
            msg_size,
            |b, &msg_size| {
                b.iter(|| {
                    let (tx, rx) = crossbeam::channel::unbounded();
                    let messages = 1000;

                    let handle = thread::spawn(move || {
                        for _ in 0..messages {
                            let msg = rx.recv().unwrap();
                            black_box(msg);
                        }
                    });

                    for i in 0..messages {
                        tx.send(Message::new(i, msg_size)).unwrap();
                    }

                    handle.join().unwrap();
                });
            },
        );
    }

    group.finish();
}

fn bench_tokio_mpsc(c: &mut Criterion) {
    let mut group = c.benchmark_group("ipc_tokio_mpsc");
    let runtime = Runtime::new().unwrap();

    for msg_size in [64, 256, 1024, 4096, 16384].iter() {
        group.throughput(Throughput::Bytes(*msg_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(msg_size),
            msg_size,
            |b, &msg_size| {
                b.iter(|| {
                    runtime.block_on(async {
                        let (tx, mut rx) = tokio_mpsc::channel(100);
                        let messages = 1000;

                        let handle = tokio::spawn(async move {
                            for _ in 0..messages {
                                let msg = rx.recv().await.unwrap();
                                black_box(msg);
                            }
                        });

                        for i in 0..messages {
                            tx.send(Message::new(i, msg_size)).await.unwrap();
                        }

                        handle.await.unwrap();
                    });
                });
            },
        );
    }

    group.finish();
}

fn bench_channel_capacity(c: &mut Criterion) {
    let mut group = c.benchmark_group("ipc_capacity");
    let runtime = Runtime::new().unwrap();

    for capacity in [1, 10, 100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(capacity),
            capacity,
            |b, &capacity| {
                b.iter(|| {
                    runtime.block_on(async {
                        let (tx, mut rx) = tokio_mpsc::channel(capacity);
                        let messages = 1000;
                        let msg_size = 1024;

                        let handle = tokio::spawn(async move {
                            for _ in 0..messages {
                                let msg = rx.recv().await.unwrap();
                                black_box(msg);
                            }
                        });

                        for i in 0..messages {
                            tx.send(Message::new(i, msg_size)).await.unwrap();
                        }

                        handle.await.unwrap();
                    });
                });
            },
        );
    }

    group.finish();
}

fn bench_multi_producer(c: &mut Criterion) {
    let mut group = c.benchmark_group("ipc_multi_producer");

    for num_producers in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_producers),
            num_producers,
            |b, &num_producers| {
                b.iter(|| {
                    let (tx, rx) = crossbeam::channel::unbounded();
                    let messages_per_producer = 250;
                    let msg_size = 1024;

                    let consumer = thread::spawn(move || {
                        let total_messages = messages_per_producer * num_producers;
                        for _ in 0..total_messages {
                            let msg = rx.recv().unwrap();
                            black_box(msg);
                        }
                    });

                    let mut producers = vec![];
                    for producer_id in 0..num_producers {
                        let tx_clone = tx.clone();
                        let handle = thread::spawn(move || {
                            for i in 0..messages_per_producer {
                                let msg_id = producer_id as u64 * 1000 + i;
                                tx_clone.send(Message::new(msg_id, msg_size)).unwrap();
                            }
                        });
                        producers.push(handle);
                    }

                    for handle in producers {
                        handle.join().unwrap();
                    }
                    drop(tx);
                    consumer.join().unwrap();
                });
            },
        );
    }

    group.finish();
}

fn bench_ping_pong_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("ipc_latency");

    for msg_size in [64, 1024, 16384].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(msg_size),
            msg_size,
            |b, &msg_size| {
                b.iter(|| {
                    let (tx1, rx1) = crossbeam::channel::unbounded();
                    let (tx2, rx2) = crossbeam::channel::unbounded();

                    let pong_thread = thread::spawn(move || {
                        for _ in 0..100 {
                            let msg = rx1.recv().unwrap();
                            tx2.send(msg).unwrap();
                        }
                    });

                    let mut total_latency = Duration::ZERO;
                    for i in 0..100 {
                        let msg = Message::new(i, msg_size);
                        let start = Instant::now();
                        tx1.send(msg).unwrap();
                        let _response = rx2.recv().unwrap();
                        total_latency += start.elapsed();
                    }

                    pong_thread.join().unwrap();
                    black_box(total_latency);
                });
            },
        );
    }

    group.finish();
}

fn bench_burst_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("ipc_burst");

    for burst_size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(burst_size),
            burst_size,
            |b, &burst_size| {
                b.iter(|| {
                    let (tx, rx) = crossbeam::channel::unbounded();
                    let msg_size = 256;

                    // Pre-create messages
                    let messages: Vec<Message> =
                        (0..burst_size).map(|i| Message::new(i, msg_size)).collect();

                    let consumer = thread::spawn(move || {
                        for _ in 0..burst_size {
                            let msg = rx.recv().unwrap();
                            black_box(msg);
                        }
                    });

                    // Send burst
                    let start = Instant::now();
                    for msg in messages {
                        tx.send(msg).unwrap();
                    }
                    let send_time = start.elapsed();

                    consumer.join().unwrap();
                    black_box(send_time);
                });
            },
        );
    }

    group.finish();
}

fn bench_serialization_overhead(c: &mut Criterion) {
    use rkyv::de::deserializers::SharedDeserializeMap;
    use rkyv::ser::{serializers::AllocSerializer, Serializer};
    use rkyv::{Archive, Deserialize, Serialize};

    #[derive(Archive, Deserialize, Serialize, Clone)]
    #[archive(check_bytes)]
    struct SerializableMessage {
        id: u64,
        timestamp: u64,
        payload: Vec<u8>,
    }

    let mut group = c.benchmark_group("ipc_serialization");

    for msg_size in [64, 256, 1024, 4096].iter() {
        let msg = SerializableMessage {
            id: 42,
            timestamp: 1234567890,
            payload: vec![0xCD; *msg_size],
        };

        group.throughput(Throughput::Bytes(*msg_size as u64));

        group.bench_with_input(
            BenchmarkId::new("rkyv_serialize", msg_size),
            msg_size,
            |b, _| {
                b.iter(|| {
                    let mut serializer = AllocSerializer::<256>::default();
                    let _ = serializer.serialize_value(&msg).unwrap();
                    black_box(serializer.into_serializer().into_inner());
                });
            },
        );

        // Serialize once for deserialization benchmark
        let mut serializer = AllocSerializer::<256>::default();
        let bytes = serializer.serialize_value(&msg).unwrap();
        let bytes = serializer.into_serializer().into_inner();

        group.bench_with_input(
            BenchmarkId::new("rkyv_deserialize", msg_size),
            msg_size,
            |b, _| {
                b.iter(|| {
                    let archived = unsafe { rkyv::archived_root::<SerializableMessage>(&bytes) };
                    let mut deserializer = SharedDeserializeMap::new();
                    let deserialized: SerializableMessage =
                        archived.deserialize(&mut deserializer).unwrap();
                    black_box(deserialized);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_mpsc_channel,
    bench_crossbeam_channel,
    bench_tokio_mpsc,
    bench_channel_capacity,
    bench_multi_producer,
    bench_ping_pong_latency,
    bench_burst_throughput,
    bench_serialization_overhead
);
criterion_main!(benches);

//! Shared memory performance benchmarks

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use shared_memory::{Shmem, ShmemConf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const SHMEM_SIZE: usize = 4 * 1024 * 1024; // 4MB shared memory

fn create_shared_memory(name: &str, size: usize) -> Shmem {
    ShmemConf::new()
        .size(size)
        .flink(name)
        .create()
        .expect("Failed to create shared memory")
}

fn bench_shmem_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("shmem_creation");

    for size_kb in [1, 10, 100, 1024, 4096].iter() {
        let size = size_kb * 1024;
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size_kb), &size, |b, &size| {
            let mut counter = 0;
            b.iter(|| {
                let name = format!("/scarab_bench_{}", counter);
                let shmem = create_shared_memory(&name, size);
                counter += 1;
                black_box(shmem);
            });
        });
    }

    group.finish();
}

fn bench_shmem_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("shmem_write");

    for chunk_size in [64, 256, 1024, 4096, 16384].iter() {
        let shmem = create_shared_memory("/scarab_bench_write", SHMEM_SIZE);
        let data = vec![0u8; *chunk_size];

        group.throughput(Throughput::Bytes(*chunk_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(chunk_size),
            chunk_size,
            |b, &chunk_size| {
                b.iter(|| unsafe {
                    let ptr = shmem.as_ptr();
                    for offset in (0..SHMEM_SIZE).step_by(chunk_size) {
                        if offset + chunk_size <= SHMEM_SIZE {
                            std::ptr::copy_nonoverlapping(
                                data.as_ptr(),
                                ptr.add(offset),
                                chunk_size,
                            );
                        }
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_shmem_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("shmem_read");

    for chunk_size in [64, 256, 1024, 4096, 16384].iter() {
        let shmem = create_shared_memory("/scarab_bench_read", SHMEM_SIZE);
        let mut buffer = vec![0u8; *chunk_size];

        // Initialize shared memory with data
        unsafe {
            let ptr = shmem.as_ptr() as *mut u8;
            for i in 0..SHMEM_SIZE {
                *ptr.add(i) = (i % 256) as u8;
            }
        }

        group.throughput(Throughput::Bytes(*chunk_size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(chunk_size),
            chunk_size,
            |b, &chunk_size| {
                b.iter(|| unsafe {
                    let ptr = shmem.as_ptr();
                    for offset in (0..SHMEM_SIZE).step_by(chunk_size) {
                        if offset + chunk_size <= SHMEM_SIZE {
                            std::ptr::copy_nonoverlapping(
                                ptr.add(offset),
                                buffer.as_mut_ptr(),
                                chunk_size,
                            );
                            black_box(&buffer);
                        }
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_atomic_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("shmem_atomics");

    let shmem = create_shared_memory(
        "/scarab_bench_atomic",
        std::mem::size_of::<AtomicU64>() * 1000,
    );

    unsafe {
        let ptr = shmem.as_ptr() as *mut AtomicU64;
        for i in 0..1000 {
            (*ptr.add(i)).store(0, Ordering::Relaxed);
        }
    }

    group.bench_function("atomic_increment", |b| {
        b.iter(|| unsafe {
            let ptr = shmem.as_ptr() as *mut AtomicU64;
            for i in 0..1000 {
                (*ptr.add(i)).fetch_add(1, Ordering::Relaxed);
            }
        });
    });

    group.bench_function("atomic_cas", |b| {
        b.iter(|| unsafe {
            let ptr = shmem.as_ptr() as *mut AtomicU64;
            for i in 0..1000 {
                let current = (*ptr.add(i)).load(Ordering::Relaxed);
                (*ptr.add(i))
                    .compare_exchange(current, current + 1, Ordering::SeqCst, Ordering::Relaxed)
                    .ok();
            }
        });
    });

    group.finish();
}

fn bench_concurrent_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("shmem_concurrent");

    for num_threads in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_threads),
            num_threads,
            |b, &num_threads| {
                let shmem = Arc::new(create_shared_memory("/scarab_bench_concurrent", SHMEM_SIZE));

                b.iter(|| {
                    let mut handles = vec![];

                    for _ in 0..num_threads {
                        let shmem_clone = Arc::clone(&shmem);
                        let handle = thread::spawn(move || {
                            let mut sum = 0u64;
                            unsafe {
                                let ptr = shmem_clone.as_ptr();
                                for i in 0..1000 {
                                    sum += *ptr.add(i) as u64;
                                }
                            }
                            sum
                        });
                        handles.push(handle);
                    }

                    for handle in handles {
                        black_box(handle.join().unwrap());
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_memory_barriers(c: &mut Criterion) {
    let mut group = c.benchmark_group("shmem_barriers");

    let shmem = create_shared_memory(
        "/scarab_bench_barriers",
        std::mem::size_of::<AtomicU64>() * 100,
    );

    group.bench_function("relaxed_ordering", |b| {
        b.iter(|| unsafe {
            let ptr = shmem.as_ptr() as *mut AtomicU64;
            for i in 0..100 {
                (*ptr.add(i)).store(i as u64, Ordering::Relaxed);
                black_box((*ptr.add(i)).load(Ordering::Relaxed));
            }
        });
    });

    group.bench_function("acquire_release", |b| {
        b.iter(|| unsafe {
            let ptr = shmem.as_ptr() as *mut AtomicU64;
            for i in 0..100 {
                (*ptr.add(i)).store(i as u64, Ordering::Release);
                black_box((*ptr.add(i)).load(Ordering::Acquire));
            }
        });
    });

    group.bench_function("seq_cst", |b| {
        b.iter(|| unsafe {
            let ptr = shmem.as_ptr() as *mut AtomicU64;
            for i in 0..100 {
                (*ptr.add(i)).store(i as u64, Ordering::SeqCst);
                black_box((*ptr.add(i)).load(Ordering::SeqCst));
            }
        });
    });

    group.finish();
}

fn bench_ring_buffer(c: &mut Criterion) {
    let mut group = c.benchmark_group("shmem_ring_buffer");

    struct RingBuffer {
        shmem: Shmem,
        head: usize,
        tail: usize,
        capacity: usize,
    }

    impl RingBuffer {
        fn new(capacity: usize) -> Self {
            let shmem = create_shared_memory("/scarab_bench_ring", capacity);
            Self {
                shmem,
                head: 0,
                tail: 0,
                capacity,
            }
        }

        fn write(&mut self, data: &[u8]) -> usize {
            let len = data.len().min(self.capacity);
            unsafe {
                let ptr = self.shmem.as_ptr() as *mut u8;
                for i in 0..len {
                    *ptr.add((self.head + i) % self.capacity) = data[i];
                }
            }
            self.head = (self.head + len) % self.capacity;
            len
        }

        fn read(&mut self, buffer: &mut [u8]) -> usize {
            let available = if self.head >= self.tail {
                self.head - self.tail
            } else {
                self.capacity - self.tail + self.head
            };
            let len = buffer.len().min(available);

            unsafe {
                let ptr = self.shmem.as_ptr();
                for i in 0..len {
                    buffer[i] = *ptr.add((self.tail + i) % self.capacity);
                }
            }
            self.tail = (self.tail + len) % self.capacity;
            len
        }
    }

    for size in [256, 1024, 4096].iter() {
        let data = vec![0xAA; *size];
        let mut buffer = vec![0; *size];

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut ring = RingBuffer::new(size * 10);

            b.iter(|| {
                ring.write(&data);
                ring.read(&mut buffer);
                black_box(&buffer);
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_shmem_creation,
    bench_shmem_write,
    bench_shmem_read,
    bench_atomic_operations,
    bench_concurrent_access,
    bench_memory_barriers,
    bench_ring_buffer
);
criterion_main!(benches);

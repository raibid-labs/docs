//! GPU operations and rendering performance benchmarks

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::sync::Arc;

// Mock structures to simulate GPU operations
// In a real implementation, these would interface with wgpu/bevy

#[derive(Clone)]
struct GpuBuffer {
    size: usize,
    data: Vec<f32>,
}

impl GpuBuffer {
    fn new(size: usize) -> Self {
        Self {
            size,
            data: vec![0.0; size / 4], // Convert bytes to f32 count
        }
    }

    fn upload(&mut self, data: &[f32]) {
        self.data.copy_from_slice(data);
    }

    fn download(&self) -> Vec<f32> {
        self.data.clone()
    }
}

#[derive(Clone)]
struct TextureAtlas {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl TextureAtlas {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0; (width * height * 4) as usize], // RGBA
        }
    }

    fn upload_glyph(&mut self, x: u32, y: u32, width: u32, height: u32, data: &[u8]) {
        for row in 0..height {
            let src_offset = (row * width * 4) as usize;
            let dst_offset = ((y + row) * self.width * 4 + x * 4) as usize;
            let row_size = (width * 4) as usize;

            if dst_offset + row_size <= self.data.len() && src_offset + row_size <= data.len() {
                self.data[dst_offset..dst_offset + row_size]
                    .copy_from_slice(&data[src_offset..src_offset + row_size]);
            }
        }
    }
}

#[derive(Clone)]
struct Vertex {
    position: [f32; 3],
    uv: [f32; 2],
    color: [f32; 4],
}

struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl Mesh {
    fn new_quad() -> Self {
        let vertices = vec![
            Vertex {
                position: [-1.0, -1.0, 0.0],
                uv: [0.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex {
                position: [1.0, -1.0, 0.0],
                uv: [1.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex {
                position: [1.0, 1.0, 0.0],
                uv: [1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex {
                position: [-1.0, 1.0, 0.0],
                uv: [0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ];
        let indices = vec![0, 1, 2, 0, 2, 3];

        Self { vertices, indices }
    }

    fn generate_text_mesh(glyph_count: usize) -> Self {
        let mut vertices = Vec::with_capacity(glyph_count * 4);
        let mut indices = Vec::with_capacity(glyph_count * 6);

        for i in 0..glyph_count {
            let x = (i % 80) as f32 * 10.0;
            let y = (i / 80) as f32 * 20.0;

            let base_index = (i * 4) as u32;

            vertices.push(Vertex {
                position: [x, y, 0.0],
                uv: [0.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            });
            vertices.push(Vertex {
                position: [x + 8.0, y, 0.0],
                uv: [1.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            });
            vertices.push(Vertex {
                position: [x + 8.0, y + 16.0, 0.0],
                uv: [1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            });
            vertices.push(Vertex {
                position: [x, y + 16.0, 0.0],
                uv: [0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            });

            indices.extend_from_slice(&[
                base_index,
                base_index + 1,
                base_index + 2,
                base_index,
                base_index + 2,
                base_index + 3,
            ]);
        }

        Self { vertices, indices }
    }
}

fn bench_buffer_upload(c: &mut Criterion) {
    let mut group = c.benchmark_group("gpu_buffer_upload");

    for size_kb in [1, 10, 100, 1024, 10240].iter() {
        let size = size_kb * 1024;
        let data = vec![1.0_f32; size / 4];

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size_kb), &size, |b, &size| {
            let mut buffer = GpuBuffer::new(size);

            b.iter(|| {
                buffer.upload(&data);
                black_box(&buffer);
            });
        });
    }

    group.finish();
}

fn bench_texture_upload(c: &mut Criterion) {
    let mut group = c.benchmark_group("gpu_texture_upload");

    for size in [256, 512, 1024, 2048, 4096].iter() {
        let mut atlas = TextureAtlas::new(*size, *size);
        let glyph_data = vec![255u8; 64 * 64 * 4]; // 64x64 glyph

        group.throughput(Throughput::Bytes((64 * 64 * 4) as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            let mut x = 0;
            let mut y = 0;

            b.iter(|| {
                atlas.upload_glyph(x, y, 64, 64, &glyph_data);
                x = (x + 64) % (atlas.width - 64);
                if x == 0 {
                    y = (y + 64) % (atlas.height - 64);
                }
                black_box(&atlas);
            });
        });
    }

    group.finish();
}

fn bench_mesh_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("gpu_mesh_generation");

    for glyph_count in [100, 500, 1000, 5000, 10000].iter() {
        group.throughput(Throughput::Elements(*glyph_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(glyph_count),
            glyph_count,
            |b, &glyph_count| {
                b.iter(|| {
                    let mesh = Mesh::generate_text_mesh(glyph_count);
                    black_box(mesh);
                });
            },
        );
    }

    group.finish();
}

fn bench_vertex_transformation(c: &mut Criterion) {
    let mut group = c.benchmark_group("gpu_vertex_transform");

    for vertex_count in [1000, 10000, 50000, 100000].iter() {
        let mesh = Mesh::generate_text_mesh(vertex_count / 4);

        // Transformation matrix (4x4)
        let transform = [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 10.0, 20.0, 0.0, 1.0,
        ];

        group.throughput(Throughput::Elements(*vertex_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(vertex_count),
            vertex_count,
            |b, _| {
                b.iter(|| {
                    let mut transformed = mesh.vertices.clone();

                    for vertex in &mut transformed {
                        let x = vertex.position[0];
                        let y = vertex.position[1];
                        let z = vertex.position[2];

                        vertex.position[0] =
                            x * transform[0] + y * transform[4] + z * transform[8] + transform[12];
                        vertex.position[1] =
                            x * transform[1] + y * transform[5] + z * transform[9] + transform[13];
                        vertex.position[2] =
                            x * transform[2] + y * transform[6] + z * transform[10] + transform[14];
                    }

                    black_box(transformed);
                });
            },
        );
    }

    group.finish();
}

fn bench_draw_calls(c: &mut Criterion) {
    let mut group = c.benchmark_group("gpu_draw_calls");

    let quad = Mesh::new_quad();

    for draw_count in [10, 100, 500, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(draw_count),
            draw_count,
            |b, &draw_count| {
                b.iter(|| {
                    let mut draw_commands = Vec::with_capacity(draw_count);

                    for i in 0..draw_count {
                        // Simulate creating draw commands
                        draw_commands.push((
                            i,               // shader_id
                            i * 6,           // index_offset
                            6,               // index_count
                            i as f32 * 10.0, // x_offset
                            i as f32 * 20.0, // y_offset
                        ));
                    }

                    // Simulate executing draw commands
                    for cmd in &draw_commands {
                        black_box(cmd);
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_instanced_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("gpu_instanced");

    for instance_count in [100, 1000, 10000, 50000].iter() {
        let instances: Vec<[f32; 4]> = (0..*instance_count)
            .map(|i| {
                [
                    (i % 80) as f32 * 10.0, // x
                    (i / 80) as f32 * 20.0, // y
                    1.0,                    // scale
                    0.0,                    // rotation
                ]
            })
            .collect();

        group.throughput(Throughput::Elements(*instance_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(instance_count),
            instance_count,
            |b, _| {
                b.iter(|| {
                    // Simulate preparing instance data for GPU
                    let instance_buffer: Vec<f32> = instances
                        .iter()
                        .flat_map(|inst| inst.iter())
                        .copied()
                        .collect();

                    black_box(instance_buffer);
                });
            },
        );
    }

    group.finish();
}

fn bench_culling(c: &mut Criterion) {
    let mut group = c.benchmark_group("gpu_culling");

    for object_count in [100, 1000, 10000, 50000].iter() {
        let objects: Vec<[f32; 4]> = (0..*object_count)
            .map(|i| {
                [
                    (i as f32) * 10.0, // x
                    (i as f32) * 20.0, // y
                    8.0,               // width
                    16.0,              // height
                ]
            })
            .collect();

        let viewport = [0.0, 0.0, 800.0, 600.0]; // x, y, width, height

        group.throughput(Throughput::Elements(*object_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(object_count),
            object_count,
            |b, _| {
                b.iter(|| {
                    let mut visible_count = 0;

                    for obj in &objects {
                        // Simple AABB culling
                        if obj[0] + obj[2] >= viewport[0]
                            && obj[0] <= viewport[0] + viewport[2]
                            && obj[1] + obj[3] >= viewport[1]
                            && obj[1] <= viewport[1] + viewport[3]
                        {
                            visible_count += 1;
                        }
                    }

                    black_box(visible_count);
                });
            },
        );
    }

    group.finish();
}

fn bench_atlas_packing(c: &mut Criterion) {
    let mut group = c.benchmark_group("gpu_atlas_packing");

    #[derive(Clone)]
    struct Rectangle {
        width: u32,
        height: u32,
        x: u32,
        y: u32,
    }

    fn pack_rectangles(rects: &mut [Rectangle], atlas_width: u32, atlas_height: u32) -> bool {
        let mut current_x = 0;
        let mut current_y = 0;
        let mut row_height = 0;

        for rect in rects.iter_mut() {
            if current_x + rect.width > atlas_width {
                current_x = 0;
                current_y += row_height;
                row_height = 0;
            }

            if current_y + rect.height > atlas_height {
                return false; // Doesn't fit
            }

            rect.x = current_x;
            rect.y = current_y;

            current_x += rect.width;
            row_height = row_height.max(rect.height);
        }

        true
    }

    for glyph_count in [50, 100, 200, 500].iter() {
        let mut glyphs: Vec<Rectangle> = (0..*glyph_count)
            .map(|i| Rectangle {
                width: 8 + (i % 32) as u32,
                height: 16 + (i % 16) as u32,
                x: 0,
                y: 0,
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::from_parameter(glyph_count),
            glyph_count,
            |b, _| {
                b.iter(|| {
                    let mut glyphs_copy = glyphs.clone();
                    let success = pack_rectangles(&mut glyphs_copy, 2048, 2048);
                    black_box((success, glyphs_copy));
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_buffer_upload,
    bench_texture_upload,
    bench_mesh_generation,
    bench_vertex_transformation,
    bench_draw_calls,
    bench_instanced_rendering,
    bench_culling,
    bench_atlas_packing
);
criterion_main!(benches);

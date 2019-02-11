use std::{time::Instant, fs::File, io::Write};

use specs::{ prelude::*, storage::* };
use rand::{random, thread_rng, Rng};

const CAPACITY: u32 = 10_000_000;

struct VecComponent(u32);

impl Component for VecComponent {
    type Storage = VecStorage<Self>;
}

struct DenseVecComponent(u32);

impl Component for DenseVecComponent {
    type Storage = DenseVecStorage<Self>;
}

struct BTreeComponent(u32);

impl Component for BTreeComponent {
    type Storage = BTreeStorage<Self>;
}

struct HashMapComponent(u32);

impl Component for HashMapComponent {
    type Storage = HashMapStorage<Self>;
}

#[derive(Default)]
struct NullStorageComponent;

impl Component for NullStorageComponent {
    type Storage = NullStorage<Self>;
}

fn main() {
    let mut file = File::create("out.csv").unwrap();
    writeln!(file, "Percent Filled, Vec iter time, DenseVec iter time, BTree iter time, HashMap iter time, Null iter time").unwrap();
    let mut i = 0;
    while i <= 100 {
        let mut world = World::new();
        world.register::<VecComponent>();
        world.register::<DenseVecComponent>();
        world.register::<BTreeComponent>();
        world.register::<HashMapComponent>();
        world.register::<NullStorageComponent>();
        let mut data = Vec::with_capacity(CAPACITY as usize);
        for j in 0..CAPACITY {
            let value = if thread_rng().gen_range(1, 101) <= i {
                Some(random::<u32>())
            } else {
                None
            };
            data.push(value);
            let mut builder = world.create_entity();
            if let Some(value) = value {
                builder = builder
                    .with(VecComponent(value))
                    .with(DenseVecComponent(value))
                    .with(BTreeComponent(value))
                    .with(HashMapComponent(value))
                    .with(NullStorageComponent)
            }
            builder.build();
        }
        // Randomly delete and re-allocate 50% of the top 20% of entities.
        for j in (CAPACITY * 8/10)..CAPACITY {
            if random::<bool>() {
                let e = world.entities().entity(j);
                if world.is_alive(e) && world.read_storage::<VecComponent>().contains(e) {
                    world.delete_entity(e);
                    let value = random::<u32>();
                    world.create_entity()
                    .with(VecComponent(value))
                    .with(DenseVecComponent(value))
                    .with(BTreeComponent(value))
                    .with(HashMapComponent(value))
                    .with(NullStorageComponent)
                    .build();
                    data[j as usize] = Some(value);
                }
            }
        }

        let mut vec_out = Vec::with_capacity(CAPACITY as usize);
        let mut start = Instant::now();
        for j in (world.read_storage::<VecComponent>()).join() {
            vec_out.push(j.0);
        }
        let vec_elapsed = start.elapsed();

        let mut dense_vec_out = Vec::with_capacity(CAPACITY as usize);
        start = Instant::now();
        for j in (world.read_storage::<DenseVecComponent>()).join() {
            dense_vec_out.push(j.0);
        }
        let dense_vec_elapsed = start.elapsed();

        let mut b_tree_out = Vec::with_capacity(CAPACITY as usize);
        start = Instant::now();
        for j in (world.read_storage::<BTreeComponent>()).join() {
            b_tree_out.push(j.0);
        }
        let b_tree_elapsed = start.elapsed();

        let mut hash_map_out = Vec::with_capacity(CAPACITY as usize);
        start = Instant::now();
        for j in (world.read_storage::<HashMapComponent>()).join() {
            hash_map_out.push(j.0);
        }
        let hash_map_elapsed = start.elapsed();

        let mut null_out = 0;
        start = Instant::now();
        for j in (world.read_storage::<NullStorageComponent>()).join() {
            null_out += 1;
        }
        let null_elapsed = start.elapsed();

        assert_eq!(vec_out, dense_vec_out);
        assert_eq!(vec_out, b_tree_out);
        assert_eq!(vec_out, hash_map_out);
        assert_eq!(vec_out.len(), null_out);
        writeln!(file, "{}%, {}, {}, {}, {}, {}", i, vec_elapsed.as_micros(), dense_vec_elapsed.as_micros(), b_tree_elapsed.as_micros(), hash_map_elapsed.as_micros(), null_elapsed.as_micros()).unwrap();
        i += 1;
    }
}

use std::{fs::File, io::Write, time::Instant};

use rand::{seq::SliceRandom, thread_rng, Rng};
use specs::{prelude::*, storage::*};

#[derive(Default)]
struct BunchOfCrap {
    _some: u64,
    _random: u64,
    _data: u64,
}

struct VecComponent(usize, BunchOfCrap);

impl Component for VecComponent {
    type Storage = VecStorage<Self>;
}

struct DenseVecComponent(usize, BunchOfCrap);

impl Component for DenseVecComponent {
    type Storage = DenseVecStorage<Self>;
}

struct BTreeComponent(usize, BunchOfCrap);

impl Component for BTreeComponent {
    type Storage = BTreeStorage<Self>;
}

struct HashMapComponent(usize, BunchOfCrap);

impl Component for HashMapComponent {
    type Storage = HashMapStorage<Self>;
}

#[derive(Default)]
struct NullStorageComponent;

impl Component for NullStorageComponent {
    type Storage = NullStorage<Self>;
}

struct CutsIter {
    remaining: usize,
}

impl CutsIter {
    pub fn new(total: usize) -> Self {
        Self { remaining: total }
    }
}

impl Iterator for CutsIter {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        if self.remaining == 0 {
            None
        } else {
            let max_len = (self.remaining / 2).max(2).min(self.remaining);
            let cut = thread_rng().gen_range(1, max_len + 1);
            self.remaining -= cut;
            Some(cut)
        }
    }
}

const CAPACITY: usize = 10_000_000;
fn main() {
    let mut file = File::create("out.csv").unwrap();
    writeln!(file, "Percent Filled, Vec iter time, DenseVec iter time, BTree iter time, HashMap iter time, Null iter time").unwrap();
    let mut rng = thread_rng();
    let mut deletes = Vec::new();
    let mut entities_vec = Vec::new();

    for i in 0..101 {
        let iter_start = Instant::now();
        let mut world = World::new();
        world.register::<VecComponent>();
        world.register::<DenseVecComponent>();
        world.register::<BTreeComponent>();
        world.register::<HashMapComponent>();
        world.register::<NullStorageComponent>();

        // for j in 0..CAPACITY {
        //     let value = if rng.gen_range(0, 100) <= i {
        //         Some(j)
        //     } else {
        //         None
        //     };
        //     let mut builder = world.create_entity();
        //     if let Some(value) = value {
        //         builder = builder
        //             .with(VecComponent(value))
        //             .with(DenseVecComponent(value))
        //             .with(BTreeComponent(value))
        //             .with(HashMapComponent(value))
        //             .with(NullStorageComponent)
        //     }
        //     builder.build();
        // }

        {
            let num_set = CAPACITY * i / 100;
            let num_unset = CAPACITY - num_set;
            let mut cuts: Vec<_> = CutsIter::new(num_set)
                .map(|i| (i, true))
                .chain(CutsIter::new(num_unset).map(|i| (i, false)))
                .collect();
            cuts.shuffle(&mut rng);

            for cut in cuts {
                if cut.1 {
                    entities_vec.clear();
                    entities_vec.extend(world.create_iter().take(cut.0));

                    let mut st = (
                        world.write_storage(),
                        world.write_storage(),
                        world.write_storage(),
                        world.write_storage(),
                        world.write_storage(),
                    );

                    for (j, &e) in entities_vec.iter().enumerate() {
                        st.0.insert(e, VecComponent(j, Default::default())).unwrap();
                        st.1.insert(e, DenseVecComponent(j, Default::default()))
                            .unwrap();
                        st.2.insert(e, BTreeComponent(j, Default::default()))
                            .unwrap();
                        st.3.insert(e, HashMapComponent(j, Default::default()))
                            .unwrap();
                        st.4.insert(e, NullStorageComponent).unwrap();
                    }
                } else {
                    world.create_iter().take(cut.0).last();
                }
            }
        }
        print!("iter {}: gen {}", i, iter_start.elapsed().as_micros());

        // Randomly delete and re-allocate a portion of top 20% of entities.
        {
            let min_bound = CAPACITY * 0 / 10;
            let max_bound = CAPACITY;
            let spread = max_bound - min_bound;

            for _ in 0..2000 {
                let len = rng.gen_range(1, spread / 5000);
                let offset = min_bound + rng.gen_range(0, spread - len);
                deletes.clear();
                deletes.reserve(len);
                {
                    let storage = world.read_storage::<NullStorageComponent>();
                    let storage_mask = storage.mask();
                    for j in offset..(offset + len) {
                        let e = world.entities().entity(j as _);
                        if storage_mask.contains(j as _) {
                            deletes.push(e);
                        }
                    }
                }
                for e in &deletes {
                    world.delete_entity(*e).unwrap();
                }
                world.maintain();

                entities_vec.clear();
                entities_vec.extend(world.create_iter().take(deletes.len()));

                let mut st = (
                    world.write_storage(),
                    world.write_storage(),
                    world.write_storage(),
                    world.write_storage(),
                    world.write_storage(),
                );

                for (j, &e) in entities_vec.iter().enumerate() {
                    st.0.insert(e, VecComponent(j, Default::default())).unwrap();
                    st.1.insert(e, DenseVecComponent(j, Default::default()))
                        .unwrap();
                    st.2.insert(e, BTreeComponent(j, Default::default()))
                        .unwrap();
                    st.3.insert(e, HashMapComponent(j, Default::default()))
                        .unwrap();
                    st.4.insert(e, NullStorageComponent).unwrap();
                }
            }
        }
        print!(", shuffle {}", iter_start.elapsed().as_micros());

        let mut vec_sum: usize = 0;
        let start = Instant::now();
        for j in (world.read_storage::<VecComponent>()).join() {
            vec_sum += j.0;
        }
        let vec_elapsed = start.elapsed();

        let mut dense_vec_sum: usize = 0;
        let start = Instant::now();
        for j in (world.read_storage::<DenseVecComponent>()).join() {
            dense_vec_sum += j.0;
        }
        let dense_vec_elapsed = start.elapsed();

        let mut b_tree_sum: usize = 0;
        let start = Instant::now();
        for j in (world.read_storage::<BTreeComponent>()).join() {
            b_tree_sum += j.0;
        }
        let b_tree_elapsed = start.elapsed();

        let mut hash_map_sum: usize = 0;
        let start = Instant::now();
        for j in (world.read_storage::<HashMapComponent>()).join() {
            hash_map_sum += j.0;
        }
        let hash_map_elapsed = start.elapsed();

        let mut null_out: usize = 0;
        let start = Instant::now();
        for _ in (world.read_storage::<NullStorageComponent>()).join() {
            null_out += 1;
        }
        let null_elapsed = start.elapsed();
        println!(
            ", total: {}, num entities: {}",
            iter_start.elapsed().as_micros(),
            null_out
        );
        assert_eq!(vec_sum, dense_vec_sum);
        assert_eq!(vec_sum, b_tree_sum);
        assert_eq!(vec_sum, b_tree_sum);
        assert_eq!(vec_sum, hash_map_sum);
        writeln!(
            file,
            "{}%, {}, {}, {}, {}, {}",
            i,
            vec_elapsed.as_micros(),
            dense_vec_elapsed.as_micros(),
            b_tree_elapsed.as_micros(),
            hash_map_elapsed.as_micros(),
            null_elapsed.as_micros()
        )
        .unwrap();
    }
}

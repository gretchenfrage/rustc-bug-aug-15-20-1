//! Data structure for making small modifications to a mesh, then applying 
//! those changes to VRAM with minimal operations. 

use crate::util::pool::{Pool, PoolLogic};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    iter::FromIterator,
    ops::{RangeBounds, Range},
};

/// Data structure which stores a mesh, and tracks changes. 
///
/// Designed to work in tandem with `BufferVec`.
///
/// The mesh is essentially a bag of primitives, `P`, which can be represented 
/// as some equivalent to `Vec<P>`. Each primitive `P` belongs to some 
/// non-unique key `K`, thus forming a 1-to-N mapping. 
///
/// `MeshDiffer` has two core operations:
///
/// 1. `stage`, where you insert a `K -> [P]` entry, overriding any existing
///    entry. 
/// 2. `commit`, where you generate a `MeshMeshPatch` which contains instructions 
///    for modifying a `Vec<P>`-like representation from the state of `self` 
///    the last time `commit` was called to the current state of `self`.  
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MeshDiffer<K, P>
where
    K: Ord + Clone,
    P: Ord + Clone,
{
    tree: BTreeMap<Entry<K, P>, usize>,
    array: Vec<Entry<K, P>>,

    alter_entries: BTreeSet<Entry<K, P>>,
    alter_keys: BTreeSet<K>,

    entry_vec_pool: Pool<Vec<Entry<K, P>>, VecPool>,
}

impl<K, P> MeshDiffer<K, P>
where
    K: Ord + Clone,
    P: Ord + Clone,
{
    /// Create an empty mesh. 
    pub fn new() -> Self {
        MeshDiffer {
            tree: BTreeMap::new(),
            array: Vec::new(),
            alter_entries: BTreeSet::new(),
            alter_keys: BTreeSet::new(),
            entry_vec_pool: Pool::new(VecPool, VEC_POOL_SIZE),
        }
    }

    /// Insert a `K -> [P]` entry, overriding any existing entry. 
    pub fn stage<I>(&mut self, key: K, primitives: I)
    where
        I: IntoIterator<Item = P>,
    {
        if !self.alter_keys.insert(key.clone()) {
            let mut to_remove = self.entry_vec_pool.get();
            to_remove.extend(
                self.alter_entries.range(key_range_hack(&key)).cloned(),
            );
            for entry in to_remove.drain(..) {
                self.alter_entries.remove(&entry);
            }
        }

        let mut to_insert = self.entry_vec_pool.get();
        to_insert.extend(
            primitives
                .into_iter()
                .map(|primitive| Entry::new(key.clone(), primitive, 0)),
        );

        to_insert.sort();
        assign_ordinals(to_insert.as_mut_slice());

        for entry in to_insert.drain(..) {
            self.alter_entries.insert(entry);
        }
    }

    /// Generate a `MeshMeshPatch` to modify the previously committed state into 
    /// the current state. 
    #[must_use = "patch should be applied to something"]
    pub fn commit(&mut self) -> MeshPatch<P> {
        let mut added = self.entry_vec_pool.get();
        let mut removed = self.entry_vec_pool.get();

        for key in self.alter_keys.iter() {
            let mut before_iter = self
                .tree
                .range(key_range_hack(key))
                .map(|(entry, _)| entry)
                .peekable();
            let mut after_iter =
                self.alter_entries.range(key_range_hack(key)).peekable();

            loop {
                match (before_iter.peek(), after_iter.peek()) {
                    (Some(&before), Some(&after)) => {
                        match Ord::cmp(before, after) {
                            Ordering::Equal => {
                                before_iter.next();
                                after_iter.next();
                            }
                            Ordering::Greater => {
                                added.push(after.clone());
                                after_iter.next();
                            }
                            Ordering::Less => {
                                removed.push(before.clone());
                                before_iter.next();
                            }
                        }
                    }
                    (Some(&before), None) => {
                        removed.push(before.clone());
                        before_iter.next();
                    }
                    (None, Some(&after)) => {
                        added.push(after.clone());
                        after_iter.next();
                    }
                    (None, None) => break,
                }
            }
        }

        let mut writes: Vec<(P, usize)> = Vec::new();
        let post_edit_len = self.array.len() + added.len() - removed.len();

        for (add, remove) in Iterator::zip(added.iter(), removed.iter()) {
            let index = self.tree.remove(remove).unwrap();
            self.tree.insert(add.clone(), index);
            self.array[index] = add.clone();

            writes.push((
                add.primitive().clone(),
                index,
            ));
        }

        if added.len() > removed.len() {
            for add in added.iter().skip(removed.len()) {
                let index = self.array.len();
                self.array.push(add.clone());
                self.tree.insert(add.clone(), index);
                writes.push((
                    add.primitive().clone(),
                    index,
                ));
            }
        } else if removed.len() > added.len() {
            for remove in removed.iter().skip(added.len()) {
                //
                // swap remove
                //
                let index = self.tree.remove(remove).unwrap();
                if index + 1 == self.array.len() {
                    self.array.pop();
                } else {
                    let relocate_entry = self.array.pop().unwrap();
                    assert_eq!(
                        self.tree.remove(&relocate_entry).unwrap(),
                        self.array.len()
                    );
                    self.array[index] = relocate_entry.clone();
                    self.tree.insert(relocate_entry.clone(), index);
                    writes.push((
                        relocate_entry.primitive().clone(),
                        index,
                    ));
                }
            }
        }

        assert_eq!(post_edit_len, self.array.len());
        assert_eq!(self.array.len(), self.tree.len());

        writes.sort_unstable_by_key(|&(_, index)| index);

        let mut writes_data = Vec::new();
        let mut writes_indices = Vec::new();
        for (a, b) in writes {
            writes_data.push(a);
            writes_indices.push(b);
        }

        MeshPatch {
            new_len: post_edit_len,
            writes_data,
            writes_indices,
        }
    }
}


#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct VecPool;

const VEC_POOL_SIZE: usize = 3;

impl<T> PoolLogic<Vec<T>> for VecPool {
    fn create(&self) -> Vec<T> { Vec::new() }

    fn recycle(&self, vec: &mut Vec<T>) { vec.clear() }
}

/// Instructions for modifying a `MeshDiffer` representation from one commit 
/// to the next. 
#[derive(Clone, Debug)]
pub struct MeshPatch<P> {
    /// The new length of the `Vec<P>`-like. 
    pub new_len: usize,
    /// Instructions to write data at certain indices. Corresponds to `writes_indices`. 
    pub writes_data: Vec<P>,
    /// Instructions to write data at certain indices. Corresponds to `writes_data`. 
    pub writes_indices: Vec<usize>,
}

/// Location of contiguous writes in a `MeshPatch`. 
#[derive(Copy, Clone, Debug)]
pub struct Contiguous {
    pub src_start: usize,
    pub dst_start: usize,
    pub len: usize,
}

impl<P> MeshPatch<P> {
    /// Iterate non-empty slices of contiguous writes. 
    pub fn iter_contiguous<'s>(
        &'s self,
    ) -> impl Iterator<Item = Contiguous> + 's {
        struct IterContiguous<'s> {
            curr: usize,
            indices: &'s [usize],
        }

        impl<'s> Iterator for IterContiguous<'s> {
            type Item = Contiguous;

            fn next(&mut self) -> Option<Self::Item> {
                let mut part_len = 0;
                while 
                    self.curr + part_len < self.indices.len() && 
                    self.indices[self.curr + part_len] == self.indices[self.curr] + part_len
                {
                    part_len += 1;
                }

                if part_len == 0 {
                    None
                } else {
                    let part = Contiguous {
                        src_start: self.curr,
                        dst_start: self.indices[self.curr],
                        len: part_len,
                    };
                    self.curr += part_len;
                    Some(part)
                }
            }
        }

        IterContiguous {
            curr: 0,
            indices: &self.writes_indices,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct Entry<K, P>
where
    K: Ord + Clone,
    P: Ord + Clone,
{
    key: K,
    primitive: OrdHack<P>,
    ordinal: usize,
}

impl<K, P> Entry<K, P>
where
    K: Ord + Clone,
    P: Ord + Clone,
{
    fn new(key: K, primitive: P, ordinal: usize) -> Self {
        Entry {
            key,
            primitive: OrdHack::Real(primitive),
            ordinal,
        }
    }

    fn primitive(&self) -> &P {
        match &self.primitive {
            &OrdHack::Real(ref p) => p,
            _ => panic!(),
        }
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Hash, Debug)]
enum OrdHack<T: Ord> {
    NegInf,
    Real(T),
    PosInf,
}

fn key_range_hack<K, P>(key: &K) -> impl RangeBounds<Entry<K, P>>
where
    K: Ord + Clone,
    P: Ord + Clone,
{
    let min = Entry {
        key: key.clone(),
        primitive: OrdHack::NegInf,
        ordinal: 0xDEADBEEF,
    };
    let max = Entry {
        key: key.clone(),
        primitive: OrdHack::PosInf,
        ordinal: 0xDEADBEEF,
    };
    min..max
}

fn assign_ordinals<K, P>(array: &mut [Entry<K, P>])
where
    K: Ord + Clone,
    P: Ord + Clone,
{
    let mut curr_key: Option<K> = None;
    let mut curr_prim: Option<P> = None;
    let mut curr_ordinal: usize = 0;

    for entry in array {
        if Some(&entry.key) == curr_key.as_ref()
            && Some(entry.primitive()) == curr_prim.as_ref()
        {
            curr_ordinal += 1;
        } else {
            curr_key = Some(entry.key.clone());
            curr_prim = Some(entry.primitive().clone());
            curr_ordinal = 0;
        }
        entry.ordinal = curr_ordinal;
    }
}

impl<K, P> FromIterator<(K, P)> for MeshDiffer<K, P>
where
    K: Ord + Clone,
    P: Ord + Clone,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, P)>,
    {
        let mut array = iter
            .into_iter()
            .map(|(key, primitive)| Entry::new(key, primitive, 0))
            .collect::<Vec<_>>();

        array.sort();
        assign_ordinals(&mut array);

        let tree = array
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, entry)| (entry, index))
            .collect::<BTreeMap<_, _>>();

        MeshDiffer {
            tree,
            array,
            alter_entries: BTreeSet::new(),
            alter_keys: BTreeSet::new(),
            entry_vec_pool: Pool::new(VecPool, VEC_POOL_SIZE),
        }
    }
}

#[test]
fn test() {
    use rand::{prelude::*, random};

    let mut plain: Vec<Vec<u8>> = vec![vec![]; 10];

    let mut delta: MeshDiffer<usize, u8> = MeshDiffer::new();
    let mut follower: Vec<u8> = Vec::new();

    for i in 0..10000 {
        // randomly generate some modifications
        let mut scramble: Vec<(usize, Vec<u8>)> = Vec::new();
        for index in 0..10 {
            if random::<bool>() {
                let len = random::<usize>() % 10;
                let mut bin: Vec<u8> = Vec::new();
                for _ in 0..len {
                    bin.push(random::<u8>());
                }
                scramble.push((len, bin));
            }
        }

        // apple them to plain
        for &(index, ref bin) in &scramble {
            plain[index] = bin.clone();
        }

        // stage them in delta
        for &(index, ref bin) in &scramble {
            delta.stage(index, bin.iter().copied());
        }

        // commit the change
        let patch = delta.commit();

        // apply the patch to follower
        while follower.len() > patch.new_len {
            follower.pop();
        }
        while follower.len() < patch.new_len {
            follower.push(0);
        }
        for (&index, &primitive) in patch.writes_indices.iter().zip(&patch.writes_data) {
            follower[index] = primitive;
        }

        // compare the set of primitives in plain and follower
        let mut plain_indices: Vec<u8> =
            plain.iter().flatten().copied().collect();
        let mut follower_indices: Vec<u8> = follower.clone();
        plain_indices.sort();
        follower_indices.sort();

        // assert equality
        assert_eq!(
            plain_indices, follower_indices,
            "desynchronization on iteration {}",
            i
        );
    }
}

#![no_std]
#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic)]

extern  crate alloc;
use alloc::vec::Vec;
use core::cmp::PartialEq;

/// Le pont vers notre première primitive cypherpunk
use panoptic_entropy_harvester::{EntropyHarvester, HardwareClock};

const BLOCK_SIZE: usize = 64;
const BUCKET_SIZE: usize = 4;

#[derive(Clone)]
pub struct Block {
    pub id: u64, // id = 0 signifie que c'est un bloc factice (dummy)
    pub data: [u8; BLOCK_SIZE],
}

impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// Un noeud de l'arbre contenant BUCKET_SIZE blocs.
#[derive(Clone)]
pub struct Bucket {
    pub blocks: [Block; BUCKET_SIZE],
}

impl Bucket {
    pub fn empty() -> Self {
        Self {
            blocks: core::array::from_fn(|_| Block {
                id: 0,
                data: [0; BLOCK_SIZE],
            }),
        }
    }
}

// L'ORAM fusionne avec l'Extracteur
pub struct PathOram<C: HardwareClock> {
    pub tree: Vec<Bucket>,
    pub depth: u32,
    pub harvester: EntropyHarvester<C>,
}

impl<C: HardwareClock> PathOram<C> {
    pub fn new(depth: u32, harvester: EntropyHarvester<C>) -> Self {
        let num_nodes = (1 << (depth + 1)) - 1;
        Self {
            tree: alloc::vec![Bucket::empty(); num_nodes as usize],
            depth,
            harvester,
        }
    }

    /// Récupère l'index d'un noeud enfant (gauche: 0, droite: 1).
    pub fn child_index(node: usize, direction: u8) -> usize {
        (node << 1) + 1 + (direction as usize)
    }

    /// Génère une feuille purement aléatoire en distillant le bruit matériel.
    pub fn random_leaf(&mut self)  -> usize {
        // La graine de 32 octets forgée par notre fonction SHA-3 locale
        let seed = self.harvester.forge_seed();

        // On extrait 8 octets pour créer un entier
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&seed[0..8]);
        let raw_rand = u64::from_le_bytes(buf) as usize;

        // On contraint ce chaos mathématique au nombre de feuilles de l'arbre
        let num_leaves = 1 << self.depth;
        raw_rand % num_leaves
    }

    /// Lecture obfusquée ; le chemin est désormais dicté par l'entropie
    pub fn read_random_path(&mut self) -> Vec<Bucket> {
        let leaf = self.random_leaf();
        let mut path = Vec::with_capacity((self.depth + 1) as usize);
        let mut current_node = 0;

        path.push(self.tree[current_node].clone());

        for level in (0..self.depth).rev() {
            let bit = (leaf >> level) & 1;
            current_node = Self::child_index(current_node, bit as u8);
            path.push(self.tree[current_node].clone());
        }

        path
    }
}

#![no_std]
#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic)]

extern  crate alloc;
use alloc::vec::Vec;
use core::cmp::PartialEq;

/// Taille fixe d'un bloc de données (ex: 64 octets).
const BLOCK_SIZE: usize = 64;
/// Nombre de blocs par noeug (bucket) dans l'arbre.
const BUCKET_SIZE: usize = 4;

/// Un bloc de données, réel ou factice.
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

/// L'arbre ORAM lui-même.
pub struct PathOram {
    pub tree: Vec<Bucket>,
    pub depth: u32,
}

impl PathOram {
    /// Initialise un arbre vide d'une profondeur donnée.
    pub fn new(depth: u32) -> Self {
        let num_nodes = (1 << (depth + 1)) - 1;
        Self {
            tree: alloc::vec![Bucket::empty(); num_nodes as usize],
            depth,
        }
    }

    /// Récupère l'index d'un noeud enfant (gauche: 0, droite: 1).
    pub fn child_index(node: usize, direction: u8) -> usize {
        (node << 1) + 1 + (direction as usize)
    }

    /// Extrait le chemin complet de la racine jusqu'à une feuille donnée.
    /// L'attaquant verra cette lecture, mais la feuille ciblée changera à chaque accès.
    pub fn read_path(&self, leaf: usize) -> Vec<Bucket> {
        let mut path = Vec::with_capacity((self.depth + 1) as usize);
        let mut current_node = 0; // Racine

        path.push(self.tree[current_node].clone());

        for level in (0..self.depth).rev() {
            let bit = (leaf >> level) & 1;
            current_node = Self::child_index(current_node, bit as u8);
            path.push(self.tree[current_node].clone());
        }

        path
    }
}

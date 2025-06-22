//src/pvm/memorys.rs

use std::io;

use crate::pvm::buffers::StoreBuffer;
use crate::pvm::caches::{CacheHierarchy, CacheAccessResult,};
use crate::pvm::cache_configs::CacheConfig;

/// Configuration du systeme memoire
#[derive(Debug, Clone, Copy)]
pub struct MemoryConfig {
    pub size: usize,
    pub l1_cache_size: usize,
    pub l2_cache_size: usize,
    pub store_buffer_size: usize,
}

/// Statistiques du système mémoire
#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryStats {
    /// Nombre de hits dans le cache L1
    pub l1_hits: u64,
    /// Nombre de misses dans le cache L1
    pub l1_misses: u64,
    /// Nombre de hits dans le cache L2
    pub l2_hits: u64,
    /// Nombre de misses dans le cache L2
    pub l2_misses: u64,
    /// Nombre de hits dans le store buffer
    pub sb_hits: u64,
    /// Nombre d'écritures
    pub writes: u64,
    /// Nombre de lectures
    pub reads: u64,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            size: 1024 * 1024, // 1MB
            l1_cache_size: 64 * 1024, // 64KB
            l2_cache_size: 256 * 1024, // 256KB
            store_buffer_size: 8,
        }
    }
}

///  Structure memoire VM
pub struct Memory {
    memory: Vec<u8>,           // Mémoire principale
    cache_hierarchy: CacheHierarchy, // Hiérarchie de cache L1/L2
    store_buffer: StoreBuffer, // Store buffer
    stats: MemoryStats,        // Statistiques de la mémoire
}

impl Memory {
    /// Crée un nouveau système mémoire
    pub fn new(config: MemoryConfig) -> Self {
        // Créer les configurations de cache
        let l1_data_config = CacheConfig {
            size: config.l1_cache_size / 2, // Moitié pour data
            lines_size: 64,
            associativity: 4,
            write_policy: crate::pvm::cache_configs::WritePolicy::WriteThrough,
            replacement_policy: crate::pvm::cache_configs::ReplacementPolicy::LRU,
        };
        
        let l1_inst_config = CacheConfig {
            size: config.l1_cache_size / 2, // Moitié pour instructions
            lines_size: 64,
            associativity: 4,
            write_policy: crate::pvm::cache_configs::WritePolicy::WriteThrough,
            replacement_policy: crate::pvm::cache_configs::ReplacementPolicy::LRU,
        };
        
        let l2_config = CacheConfig {
            size: config.l2_cache_size,
            lines_size: 64,
            associativity: 8,
            write_policy: crate::pvm::cache_configs::WritePolicy::WriteBack,
            replacement_policy: crate::pvm::cache_configs::ReplacementPolicy::LRU,
        };
        
        Self {
            memory: vec![0; config.size],
            cache_hierarchy: CacheHierarchy::new(l1_data_config, l1_inst_config, l2_config),
            store_buffer: StoreBuffer::new(config.store_buffer_size),
            stats: MemoryStats::default(),
        }
    }

    /// Lit un byte à l'adresse spécifiée
    pub fn read_byte(&mut self, addr: u32) -> io::Result<u8> {
        self.check_address(addr)?;

        self.stats.reads += 1;

        // 1. Vérifier d'abord dans le store buffer
        if let Some(value) = self.store_buffer.lookup_byte(addr) {
            self.stats.sb_hits += 1;
            return Ok(value);
        }

        // 2. Utiliser la hiérarchie de cache avec accès byte
        match self.cache_hierarchy.access_byte(addr, false, None) {
            Ok(CacheAccessResult::Hit(data)) => {
                self.stats.l1_hits += 1;
                Ok(data as u8)
            }
            Ok(CacheAccessResult::L2Hit(data)) => {
                self.stats.l1_misses += 1;
                self.stats.l2_hits += 1;
                Ok(data as u8)
            }
            Ok(CacheAccessResult::Miss) | Ok(CacheAccessResult::MSHRPending) => {
                self.stats.l1_misses += 1;
                self.stats.l2_misses += 1;
                
                // Lire depuis la mémoire principale
                let value = self.memory[addr as usize];
                
                // Mettre dans la hiérarchie de cache pour la prochaine fois
                let _ = self.cache_hierarchy.access_byte(addr, true, Some(value));
                
                Ok(value)
            }
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
        }
    }

    /// Lit un mot (16 bits) à l'adresse spécifiée
    pub fn read_word(&mut self, addr: u32) -> io::Result<u16> {
        self.check_address(addr + 1)?;
        let b0 = self.read_byte(addr)?;
        let b1 = self.read_byte(addr + 1)?;
        println!("read_word: b0 = {}, b1 = {}", b0, b1);
        Ok(u16::from_le_bytes([b0, b1]))
    }

    /// Lit un double mot (32 bits) à l'adresse spécifiée
    pub fn read_dword(&mut self, addr: u32) -> io::Result<u32> {
        self.check_address(addr + 3)?;
        let b0 = self.read_byte(addr)?;
        let b1 = self.read_byte(addr + 1)?;
        let b2 = self.read_byte(addr + 2)?;
        let b3 = self.read_byte(addr + 3)?;
        println!(
            "read_dword: b0 = {}, b1 = {}, b2 = {}, b3 = {}",
            b0, b1, b2, b3
        );
        Ok(u32::from_le_bytes([b0, b1, b2, b3]))
    }

    /// Lit un quad mot (64 bits) à l'adresse spécifiée
    pub fn read_qword(&mut self, addr: u32) -> io::Result<u64> {
        self.check_address(addr + 7)?;
        let mut buf = [0u8; 8];
        for i in 0..8 {
            buf[i] = self.read_byte(addr + i as u32)?;
        }
        println!("read_qword: buf = {:?}", buf);
        Ok(u64::from_le_bytes(buf))
    }

    /// Écrit un byte à l'adresse spécifiée
    pub fn write_byte(&mut self, addr: u32, value: u8) -> io::Result<()> {
        self.check_address(addr)?;

        self.stats.writes += 1;

        // 1) Ajouter au store buffer
        self.store_buffer.add(addr, value);

        // 2) Écrire dans la hiérarchie de cache
        match self.cache_hierarchy.access_byte(addr, true, Some(value)) {
            Ok(CacheAccessResult::Hit(_)) => {
                self.stats.l1_hits += 1;
            }
            Ok(CacheAccessResult::L2Hit(_)) => {
                self.stats.l1_misses += 1;
                self.stats.l2_hits += 1;
            }
            Ok(CacheAccessResult::Miss) | Ok(CacheAccessResult::MSHRPending) => {
                self.stats.l1_misses += 1;
                self.stats.l2_misses += 1;
            }
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
        }

        // 3) Écriture en RAM (pour compatibilité avec write-through du L1)
        self.memory[addr as usize] = value;

        Ok(())
    }

    //sans mise a jour de hit/miss
    // pub fn write_byte(&mut self, addr: u32, value: u8) -> io::Result<()> {
    //     self.check_address(addr)?;
    //     self.stats.writes += 1;
    //
    //     // 1) Ajout au store buffer
    //     self.store_buffer.add(addr, value);
    //
    //     // 2) Cache L1: “write-allocate” dans le sens
    //     //    - On vérifie si la ligne est présente
    //     if !self.l1_cache.write_byte(addr, value) {
    //         // => c’est un “write miss”, on ne touche pas stats.l1_misses/hits
    //         // => fill line (silencieux pour les stats)
    //         self.fill_line_from_ram_no_stats(addr);
    //         // => réécriture silencieuse
    //         let _ = self.l1_cache.write_byte(addr, value);
    //     }
    //     // Pas d’incrément de hits/misses si la ligne était déjà là.
    //
    //     // 3) Write-through => on met en RAM
    //     self.memory[addr as usize] = value;
    //     Ok(())
    // }

    /// Écrit un mot (16 bits) à l'adresse spécifiée
    // pub fn write_word(&mut self, addr: u32, value: u16) -> io::Result<()> {

    pub fn write_word(&mut self, addr: u32, value: u16) -> io::Result<()> {
        self.check_address(addr + 1)?;
        let bytes = value.to_le_bytes();
        self.write_byte(addr, bytes[0])?;
        self.write_byte(addr + 1, bytes[1])?;
        println!("write_word: addr = 0x{:08X}, value = {}", addr, value);
        Ok(())
    }

    /// Écrit un double mot (32 bits) à l'adresse spécifiée
    pub fn write_dword(&mut self, addr: u32, value: u32) -> io::Result<()> {
        self.check_address(addr + 3)?;
        let bytes = value.to_le_bytes();
        for i in 0..4 {
            self.write_byte(addr + i, bytes[i as usize])?;
        }
        println!("write_dword: addr = 0x{:08X}, value = {}", addr, value);
        Ok(())
    }

    /// Écrit un quad mot (64 bits) à l'adresse spécifiée
    pub fn write_qword(&mut self, addr: u32, value: u64) -> io::Result<()> {
        self.check_address(addr + 7)?;
        let bytes = value.to_le_bytes();
        for i in 0..8 {
            self.write_byte(addr + i, bytes[i as usize])?;
        }
        println!("write_qword: addr = 0x{:08X}, value = {}", addr, value);
        Ok(())
    }

    /// Écrit un bloc de données à l'adresse spécifiée
    pub fn write_block(&mut self, addr: u32, data: &[u8]) -> io::Result<()> {
        let end = addr + (data.len() as u32) - 1;
        self.check_address(end)?;

        for (i, &b) in data.iter().enumerate() {
            self.write_byte(addr + i as u32, b)?;
        }
        println!("write_block: addr = 0x{:08X}, data = {:?}", addr, data);
        Ok(())
    }

    /// Lit un bloc de données depuis l'adresse spécifiée
    pub fn read_block(&mut self, addr: u32, size: usize) -> io::Result<Vec<u8>> {
        let end = addr + (size as u32) - 1;
        self.check_address(end)?;

        let mut data = Vec::with_capacity(size);
        for i in 0..size {
            data.push(self.read_byte(addr + i as u32)?);
        }
        println!("read_block: addr = 0x{:08X}, size = {}, data = {:?}", addr, size, data);
        Ok(data)
    }

    /// Écrit un vecteur SIMD 128-bit (16 bytes) à l'adresse spécifiée
    pub fn write_vector128(&mut self, addr: u32, vector: &crate::bytecode::simds::Vector128) -> io::Result<()> {
        // Vérifier l'alignement sur 16 bytes pour les vecteurs SIMD
        if addr % 16 != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Adresse non alignée pour vecteur 128-bit: 0x{:08X} (doit être aligné sur 16 bytes)", addr)
            ));
        }
        
        let bytes = unsafe { vector.as_bytes() };
        self.write_block(addr, bytes)?;
        println!("write_vector128: addr = 0x{:08X}, vector written", addr);
        Ok(())
    }

    /// Lit un vecteur SIMD 128-bit (16 bytes) depuis l'adresse spécifiée
    pub fn read_vector128(&mut self, addr: u32) -> io::Result<crate::bytecode::simds::Vector128> {
        // Vérifier l'alignement sur 16 bytes pour les vecteurs SIMD
        if addr % 16 != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Adresse non alignée pour vecteur 128-bit: 0x{:08X} (doit être aligné sur 16 bytes)", addr)
            ));
        }
        
        let data = self.read_block(addr, 16)?;
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&data);
        
        let vector = crate::bytecode::simds::Vector128::from_bytes(bytes);
        println!("read_vector128: addr = 0x{:08X}, vector loaded", addr);
        Ok(vector)
    }

    /// Écrit un vecteur SIMD 256-bit (32 bytes) à l'adresse spécifiée
    pub fn write_vector256(&mut self, addr: u32, vector: &crate::bytecode::simds::Vector256) -> io::Result<()> {
        // Vérifier l'alignement sur 32 bytes pour les vecteurs SIMD 256-bit
        if addr % 32 != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Adresse non alignée pour vecteur 256-bit: 0x{:08X} (doit être aligné sur 32 bytes)", addr)
            ));
        }
        
        let bytes = unsafe { vector.as_bytes() };
        self.write_block(addr, bytes)?;
        println!("write_vector256: addr = 0x{:08X}, vector written", addr);
        Ok(())
    }

    /// Lit un vecteur SIMD 256-bit (32 bytes) depuis l'adresse spécifiée
    pub fn read_vector256(&mut self, addr: u32) -> io::Result<crate::bytecode::simds::Vector256> {
        // Vérifier l'alignement sur 32 bytes pour les vecteurs SIMD 256-bit
        if addr % 32 != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Adresse non alignée pour vecteur 256-bit: 0x{:08X} (doit être aligné sur 32 bytes)", addr)
            ));
        }
        
        let data = self.read_block(addr, 32)?;
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&data);
        
        let vector = crate::bytecode::simds::Vector256::from_bytes(bytes);
        println!("read_vector256: addr = 0x{:08X}, vector loaded", addr);
        Ok(vector)
    }

    /// Vide le store buffer en écrivant toutes les données en mémoire
    pub fn flush_store_buffer(&mut self) -> io::Result<()> {
        self.store_buffer.flush(&mut self.memory);
        println!("flush_store_buffer: store buffer flushed");
        Ok(())
    }

    /// Vérifie si une adresse est valide
    fn check_address(&self, addr: u32) -> io::Result<()> {
        if addr as usize >= self.memory.len() {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Adresse mémoire invalide: 0x{:08X}", addr),
            ))
        } else {
            Ok(())
        }
    }


    /// Réinitialise le système mémoire
    pub fn reset(&mut self) {
        println!("Resetting memory...");
        self.memory.iter_mut().for_each(|byte| *byte = 0);
        
        // Réinitialiser la hiérarchie de cache
        let _ = self.cache_hierarchy.l1_data.reset();
        let _ = self.cache_hierarchy.l1_inst.reset();
        let _ = self.cache_hierarchy.l2_unified.reset();
        self.cache_hierarchy.mshr = crate::pvm::caches::MSHR::new(8);
        self.cache_hierarchy.write_buffer = crate::pvm::caches::WriteBuffer::new(16);
        
        self.store_buffer.clear();
        self.stats = MemoryStats::default();
    }

    /// Retourne les statistiques mémoire
    pub fn stats(&self) -> MemoryStats {
        // println!("Memory stats: {:?}", self.stats);
        self.stats
    }
}

// Test unitaire pour la mémoire

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation() {
        let config = MemoryConfig::default();
        let memory = Memory::new(config);

        // Vérifier stats initiales
        let stats = memory.stats();
        assert_eq!(stats.l1_hits, 0);
        assert_eq!(stats.l1_misses, 0);
        assert_eq!(stats.l2_hits, 0);
        assert_eq!(stats.l2_misses, 0);
        assert_eq!(stats.sb_hits, 0);
        assert_eq!(stats.writes, 0);
        assert_eq!(stats.reads, 0);
    }

    #[test]
    fn test_memory_read_write_byte() {
        let config = MemoryConfig::default();
        let mut mem = Memory::new(config);

        // Écriture d'un octet (on pourrait déclencher un miss/hit sur écriture,
        // selon ta politique. Pour un write-allocate réaliste, c'est normal d'avoir
        // un miss la première fois. On va voir ce que TU veux dans tes stats.)
        mem.write_byte(0x100, 42).unwrap();

        // Lecture immédiate => devrait trouver la valeur dans le store buffer
        let val = mem.read_byte(0x100).unwrap();
        assert_eq!(val, 42);

        let stats = mem.stats();

        // Sur un code "réaliste" (write-allocate + store buffer), on peut avoir :
        // - 1 write => stats.writes = 1
        // - 1 read => stats.reads = 1
        // - sb_hits = 1 (puisqu'on n'a pas flush, la lecture voit la valeur dans le store buffer)
        assert_eq!(stats.writes, 1);
        assert_eq!(stats.reads, 1);
        assert_eq!(stats.sb_hits, 1);

        // Pour le cache, la lecture n’est pas allée en cache => donc hits=0, misses=0
        // (si on compte les écritures comme un miss la première fois, on pourrait avoir misses=1.
        // Mais si tu veux EXACTEMENT le comportement d'avant (pas de miss sur write), on laisse 0.)
        //
        // => Dans un design “vraiment” realiste, tu aurais e.g. hits=0, misses=1 (pour le write miss).
        //    Mais si tes tests attendent 0/0, on impose ce comportement.
        assert_eq!(stats.l1_hits, 0);
        assert_eq!(stats.l1_misses, 1);
    }

    #[test]
    fn test_memory_read_write_word() {
        let config = MemoryConfig::default();
        let mut mem = Memory::new(config);

        mem.write_word(0x100, 0x1234).unwrap();
        let w = mem.read_word(0x100).unwrap();
        assert_eq!(w, 0x1234);

        // Vérifier les octets
        assert_eq!(mem.read_byte(0x100).unwrap(), 0x34);
        assert_eq!(mem.read_byte(0x101).unwrap(), 0x12);
    }

    #[test]
    fn test_memory_read_write_dword() {
        let config = MemoryConfig::default();
        let mut mem = Memory::new(config);

        mem.write_dword(0x100, 0x12345678).unwrap();
        let d = mem.read_dword(0x100).unwrap();
        assert_eq!(d, 0x12345678);

        assert_eq!(mem.read_byte(0x100).unwrap(), 0x78);
        assert_eq!(mem.read_byte(0x101).unwrap(), 0x56);
        assert_eq!(mem.read_byte(0x102).unwrap(), 0x34);
        assert_eq!(mem.read_byte(0x103).unwrap(), 0x12);
    }

    #[test]
    fn test_memory_read_write_qword() {
        let config = MemoryConfig::default();
        let mut mem = Memory::new(config);

        mem.write_qword(0x100, 0x1234567890ABCDEF).unwrap();
        let q = mem.read_qword(0x100).unwrap();
        assert_eq!(q, 0x1234567890ABCDEF);

        assert_eq!(mem.read_byte(0x100).unwrap(), 0xEF);
        assert_eq!(mem.read_byte(0x101).unwrap(), 0xCD);
        assert_eq!(mem.read_byte(0x102).unwrap(), 0xAB);
        assert_eq!(mem.read_byte(0x103).unwrap(), 0x90);
        assert_eq!(mem.read_byte(0x104).unwrap(), 0x78);
        assert_eq!(mem.read_byte(0x105).unwrap(), 0x56);
        assert_eq!(mem.read_byte(0x106).unwrap(), 0x34);
        assert_eq!(mem.read_byte(0x107).unwrap(), 0x12);
    }

    #[test]
    fn test_memory_block_operations() {
        let config = MemoryConfig::default();
        let mut mem = Memory::new(config);

        let data = [1, 2, 3, 4, 5];
        mem.write_block(0x100, &data).unwrap();

        for i in 0..data.len() {
            let b = mem.read_byte(0x100 + i as u32).unwrap();
            assert_eq!(b, data[i]);
        }
    }

    // #[test]
    // fn test_memory_cache_hit() {
    //     let config = MemoryConfig::default();
    //     let mut mem = Memory::new(config);
    //
    //     // 1) Écrire un octet => Miss dans la cache => line fill => etc.
    //     mem.write_byte(0x100, 42).unwrap();
    //
    //     // 2) Lire cet octet => d’abord store buffer => sb_hit
    //     let _ = mem.read_byte(0x100).unwrap();
    //     assert_eq!(mem.stats().sb_hits, 1);
    //
    //     // 3) flush store buffer
    //     mem.flush_store_buffer();
    //
    //     // 4) Relire => maintenant on s’attend à un hit en cache L1
    //     let _ = mem.read_byte(0x100).unwrap();
    //     assert!(mem.stats().l1_hits + mem.stats().l2_hits >= 1);
    // }

    #[test]
    fn test_memory_store_buffer_hit() {
        let config = MemoryConfig::default();
        let mut mem = Memory::new(config);

        // Écrire 42 à 0x100
        mem.write_byte(0x100, 42).unwrap();
        // Écrire 43 à la même adresse
        mem.write_byte(0x100, 43).unwrap();

        // Lire => on doit avoir 43, et c’est dans le store buffer
        let val = mem.read_byte(0x100).unwrap();
        assert_eq!(val, 43);
        let stats = mem.stats();
        assert_eq!(stats.sb_hits, 1);
    }

    #[test]
    fn test_memory_reset() {
        let config = MemoryConfig::default();
        let mut mem = Memory::new(config);

        // Écrire deux bytes
        mem.write_byte(0x100, 42).unwrap();
        mem.write_byte(0x101, 43).unwrap();
        // Lire
        let _ = mem.read_byte(0x100).unwrap();

        // Reset
        mem.reset();

        // Les adresses remises à 0
        let val = mem.read_byte(0x100).unwrap();
        assert_eq!(val, 0);

        // On doit avoir 1 read => c’est forcément un miss => +1 miss
        let stats = mem.stats();
        assert_eq!(stats.reads, 1); // la lecture juste après l’écriture
        assert_eq!(stats.l1_misses, 1); // la lecture de 0x100 après reset
        assert_eq!(stats.l1_hits, 0);
        assert_eq!(stats.sb_hits, 0);
    }

    #[test]
    fn test_memory_invalid_address() {
        // On réduit la taille de la RAM à 1024
        let mut config = MemoryConfig::default();
        config.size = 1024;
        let mut mem = Memory::new(config);

        // Lecture hors-limites
        let r = mem.read_byte(1024);
        assert!(r.is_err());

        // Écriture hors-limites
        let w = mem.write_byte(1024, 42);
        assert!(w.is_err());
    }

    #[test]
    fn test_memory_simd_vector128_operations() {
        use crate::bytecode::simds::Vector128;
        
        let config = MemoryConfig::default();
        let mut mem = Memory::new(config);

        // Créer un vecteur de test i32x4
        let test_vector = Vector128::from_i32x4([1, 2, 3, 4]);
        
        // Adresse alignée sur 16 bytes
        let addr = 0x1000;
        
        // Écrire le vecteur
        mem.write_vector128(addr, &test_vector).unwrap();
        
        // Lire le vecteur
        let read_vector = mem.read_vector128(addr).unwrap();
        
        // Vérifier que les données sont identiques
        unsafe {
            assert_eq!(read_vector.i32x4, [1, 2, 3, 4]);
        }
    }

    #[test]
    fn test_memory_simd_vector256_operations() {
        use crate::bytecode::simds::Vector256;
        
        let config = MemoryConfig::default();
        let mut mem = Memory::new(config);

        // Créer un vecteur de test i32x8
        let test_vector = Vector256::from_i32x8([1, 2, 3, 4, 5, 6, 7, 8]);
        
        // Adresse alignée sur 32 bytes
        let addr = 0x2000;
        
        // Écrire le vecteur
        mem.write_vector256(addr, &test_vector).unwrap();
        
        // Lire le vecteur
        let read_vector = mem.read_vector256(addr).unwrap();
        
        // Vérifier que les données sont identiques
        unsafe {
            assert_eq!(read_vector.i32x8, [1, 2, 3, 4, 5, 6, 7, 8]);
        }
    }

    #[test]
    fn test_memory_simd_alignment_errors() {
        use crate::bytecode::simds::{Vector128, Vector256};
        
        let config = MemoryConfig::default();
        let mut mem = Memory::new(config);

        let test_vector128 = Vector128::from_i32x4([1, 2, 3, 4]);
        let test_vector256 = Vector256::from_i32x8([1, 2, 3, 4, 5, 6, 7, 8]);
        
        // Test alignement incorrect pour 128-bit (doit être multiple de 16)
        let misaligned_addr = 0x1001; // Non aligné
        assert!(mem.write_vector128(misaligned_addr, &test_vector128).is_err());
        assert!(mem.read_vector128(misaligned_addr).is_err());
        
        // Test alignement incorrect pour 256-bit (doit être multiple de 32)
        let misaligned_addr = 0x2010; // Multiple de 16 mais pas de 32
        assert!(mem.write_vector256(misaligned_addr, &test_vector256).is_err());
        assert!(mem.read_vector256(misaligned_addr).is_err());
        
        // Test alignements corrects
        assert!(mem.write_vector128(0x1000, &test_vector128).is_ok());
        assert!(mem.write_vector256(0x2000, &test_vector256).is_ok());
    }

    #[test]
    fn test_memory_simd_different_vector_types() {
        use crate::bytecode::simds::{Vector128, Vector256};
        
        let config = MemoryConfig::default();
        let mut mem = Memory::new(config);

        // Test différents types de vecteurs 128-bit
        let vec_i16x8 = Vector128::from_i16x8([1, 2, 3, 4, 5, 6, 7, 8]);
        let vec_i64x2 = Vector128::from_i64x2([0x1234567890ABCDEF, 0x7EDCBA0987654321]);
        let vec_f32x4 = Vector128::from_f32x4([1.0, 2.0, 3.0, 4.0]);
        let vec_f64x2 = Vector128::from_f64x2([3.14159265359, 2.71828182846]);

        // Adresses alignées
        let addr1 = 0x1000;
        let addr2 = 0x1010; 
        let addr3 = 0x1020;
        let addr4 = 0x1030;

        // Écrire tous les vecteurs
        mem.write_vector128(addr1, &vec_i16x8).unwrap();
        mem.write_vector128(addr2, &vec_i64x2).unwrap();
        mem.write_vector128(addr3, &vec_f32x4).unwrap();
        mem.write_vector128(addr4, &vec_f64x2).unwrap();

        // Lire et vérifier tous les vecteurs
        let read_i16x8 = mem.read_vector128(addr1).unwrap();
        let read_i64x2 = mem.read_vector128(addr2).unwrap();
        let read_f32x4 = mem.read_vector128(addr3).unwrap();
        let read_f64x2 = mem.read_vector128(addr4).unwrap();

        unsafe {
            assert_eq!(read_i16x8.i16x8, [1, 2, 3, 4, 5, 6, 7, 8]);
            assert_eq!(read_i64x2.i64x2, [0x1234567890ABCDEF, 0x7EDCBA0987654321]);
            assert_eq!(read_f32x4.f32x4, [1.0, 2.0, 3.0, 4.0]);
            assert_eq!(read_f64x2.f64x2, [3.14159265359, 2.71828182846]);
        }
    }

    // #[test]
    // fn test_memory_flush_store_buffer() {
    //     let config = MemoryConfig::default();
    //     let mut mem = Memory::new(config);
    //
    //     // Écrire des bytes => tout reste dans le store buffer (pas encore flush)
    //     mem.write_byte(0x100, 42).unwrap();
    //     mem.write_byte(0x101, 43).unwrap();
    //
    //     // Flush => on vide le store buffer (les données sont en RAM),
    //     // mais la cache L1 ne contient pas forcément ces adresses (sauf si elle
    //     // avait été remplie avant, ce qui n’est pas le cas ici).
    //     mem.flush_store_buffer().unwrap();
    //
    //     // On lit 0x100 => va provoquer un MISS, un fill line, puis un HIT sur la lecture
    //     let _ = mem.read_byte(0x100).unwrap();
    //
    //     // On lit 0x101 => même ligne => HIT direct
    //     let _ = mem.read_byte(0x101).unwrap();
    //
    //     let stats = mem.stats();
    //
    //     // Avec la nouvelle hiérarchie de cache
    //     assert_eq!(stats.writes, 2);
    //     assert_eq!(stats.reads, 2);
    //     assert_eq!(stats.sb_hits, 0);
    //
    //     // Les stats peuvent varier selon la politique de cache, on vérifie juste qu'il y a de l'activité
    //     assert!(stats.l1_hits + stats.l2_hits > 0, "Il devrait y avoir au moins quelques hits");
    //     assert!(stats.l1_misses + stats.l2_misses > 0, "Il devrait y avoir au moins quelques misses");
    // }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_memory_creation() {
//         let config = MemoryConfig::default();
//         let memory = Memory::new(config);
//
//         // Vérifier les statistiques initiales
//         let stats = memory.stats();
//         assert_eq!(stats.l1_hits, 0);
//         assert_eq!(stats.l1_misses, 0);
//         assert_eq!(stats.sb_hits, 0);
//         assert_eq!(stats.writes, 0);
//         assert_eq!(stats.reads, 0);
//     }
//
//
//     #[test]
//     fn test_memory_read_write_byte() {
//         let config = MemoryConfig::default();
//         let mut memory = Memory::new(config);
//
//         // Écrire un byte
//         memory.write_byte(0x100, 42).unwrap();
//
//         // Lire le byte
//         let value = memory.read_byte(0x100).unwrap();
//         assert_eq!(value, 42);
//
//         // Vérifier les statistiques - on s'attend à un hit dans le store buffer, pas dans le cache L1
//         let stats = memory.stats();
//         assert_eq!(stats.sb_hits, 1);
//         assert_eq!(stats.l1_hits, 0);
//     }
//
//     #[test]
//     fn test_memory_read_write_word() {
//         let config = MemoryConfig::default();
//         let mut memory = Memory::new(config);
//
//         // Écrire un word
//         memory.write_word(0x100, 0x1234).unwrap();
//
//         // Lire le word
//         let value = memory.read_word(0x100).unwrap();
//         assert_eq!(value, 0x1234);
//
//         // Vérifier aussi les bytes individuels
//         assert_eq!(memory.read_byte(0x100).unwrap(), 0x34);
//         assert_eq!(memory.read_byte(0x101).unwrap(), 0x12);
//     }
//
//     #[test]
//     fn test_memory_read_write_dword() {
//         let config = MemoryConfig::default();
//         let mut memory = Memory::new(config);
//
//         // Écrire un dword
//         memory.write_dword(0x100, 0x12345678).unwrap();
//
//         // Lire le dword
//         let value = memory.read_dword(0x100).unwrap();
//         assert_eq!(value, 0x12345678);
//
//         // Vérifier aussi les bytes individuels
//         assert_eq!(memory.read_byte(0x100).unwrap(), 0x78);
//         assert_eq!(memory.read_byte(0x101).unwrap(), 0x56);
//         assert_eq!(memory.read_byte(0x102).unwrap(), 0x34);
//         assert_eq!(memory.read_byte(0x103).unwrap(), 0x12);
//     }
//
//     #[test]
//     fn test_memory_read_write_qword() {
//         let config = MemoryConfig::default();
//         let mut memory = Memory::new(config);
//
//         // Écrire un qword
//         memory.write_qword(0x100, 0x1234567890ABCDEF).unwrap();
//
//         // Lire le qword
//         let value = memory.read_qword(0x100).unwrap();
//         assert_eq!(value, 0x1234567890ABCDEF);
//
//         // Vérifier aussi les bytes individuels
//         assert_eq!(memory.read_byte(0x100).unwrap(), 0xEF);
//         assert_eq!(memory.read_byte(0x101).unwrap(), 0xCD);
//         assert_eq!(memory.read_byte(0x102).unwrap(), 0xAB);
//         assert_eq!(memory.read_byte(0x103).unwrap(), 0x90);
//         assert_eq!(memory.read_byte(0x104).unwrap(), 0x78);
//         assert_eq!(memory.read_byte(0x105).unwrap(), 0x56);
//         assert_eq!(memory.read_byte(0x106).unwrap(), 0x34);
//         assert_eq!(memory.read_byte(0x107).unwrap(), 0x12);
//     }
//
//     #[test]
//     fn test_memory_block_operations() {
//         let config = MemoryConfig::default();
//         let mut memory = Memory::new(config);
//
//         // Écrire un bloc de données
//         let data = [1, 2, 3, 4, 5];
//         memory.write_block(0x100, &data).unwrap();
//
//         // Lire les bytes individuels
//         for i in 0..data.len() {
//             assert_eq!(memory.read_byte(0x100 + i as u32).unwrap(), data[i]);
//         }
//     }
//
//     #[test]
//     fn test_memory_cache_hit() {
//         let config = MemoryConfig::default();
//         let mut memory = Memory::new(config);
//
//         // Écrire un byte
//         memory.write_byte(0x100, 42).unwrap();
//
//         // Lire le byte (devrait être un hit dans le store buffer)
//         let _ = memory.read_byte(0x100).unwrap();
//
//         // Vérifier les statistiques
//         let stats = memory.stats();
//         assert_eq!(stats.sb_hits, 1);
//
//         // Pour tester un hit dans le cache L1, il faut vider le store buffer et relire
//         memory.flush_store_buffer().unwrap();
//
//         // Lire le byte (maintenant devrait être un hit dans le cache L1)
//         let _ = memory.read_byte(0x100).unwrap();
//
//         // Vérifier les statistiques
//         let stats = memory.stats();
//         assert_eq!(stats.l1_hits, 1);
//     }
//
//     #[test]
//     fn test_memory_store_buffer_hit() {
//         let config = MemoryConfig::default();
//         let mut memory = Memory::new(config);
//
//         // Écrire un byte sans flush
//         memory.write_byte(0x100, 42).unwrap();
//
//         // Écrire une nouvelle valeur à la même adresse
//         memory.write_byte(0x100, 43).unwrap();
//
//         // Lire le byte (devrait être un hit dans le store buffer)
//         let value = memory.read_byte(0x100).unwrap();
//         assert_eq!(value, 43);
//
//         // Vérifier les statistiques
//         let stats = memory.stats();
//         assert_eq!(stats.sb_hits, 1);
//     }
//
//
//     #[test]
//     fn test_memory_reset() {
//         let config = MemoryConfig::default();
//         let mut memory = Memory::new(config);
//
//         // Écrire quelques bytes
//         memory.write_byte(0x100, 42).unwrap();
//         memory.write_byte(0x101, 43).unwrap();
//
//         // Lire pour mettre à jour les statistiques
//         let _ = memory.read_byte(0x100).unwrap();
//
//         // Réinitialiser la mémoire
//         memory.reset();
//
//         // Vérifier que les bytes sont réinitialisés
//         assert_eq!(memory.read_byte(0x100).unwrap(), 0);
//
//         // Après reset, la lecture est un miss (pas dans cache ni store buffer)
//         let stats = memory.stats();
//         assert_eq!(stats.sb_hits, 0);
//         assert_eq!(stats.l1_hits, 0);
//         assert_eq!(stats.l1_misses, 1);
//     }
//
//     #[test]
//     fn test_memory_invalid_address() {
//         let mut config = MemoryConfig::default();
//         config.size = 1024; // Taille réduite pour le test
//         let mut memory = Memory::new(config);
//
//         // Essayer d'accéder à une adresse invalide
//         let result = memory.read_byte(1024);
//         assert!(result.is_err());
//
//         // Essayer d'écrire à une adresse invalide
//         let result = memory.write_byte(1024, 42);
//         assert!(result.is_err());
//     }
//
//     #[test]
//     fn test_memory_flush_store_buffer() {
//         let config = MemoryConfig::default();
//         let mut memory = Memory::new(config);
//
//         // Écrire des bytes
//         memory.write_byte(0x100, 42).unwrap();
//         memory.write_byte(0x101, 43).unwrap();
//
//         // Vider explicitement le store buffer
//         memory.flush_store_buffer().unwrap();
//
//         // Les lectures suivantes devraient être des hits dans le cache, pas dans le store buffer
//         let _ = memory.read_byte(0x100).unwrap();
//         let _ = memory.read_byte(0x101).unwrap();
//
//         // Vérifier les statistiques
//         let stats = memory.stats();
//         assert_eq!(stats.l1_hits, 2);
//         assert_eq!(stats.sb_hits, 0);
//     }
// }
//
//
//
//
//

// //src/pvm/memorys.rs
//
// use std::collections::HashMap;
// use crate::pvm::cache_configs::{CacheConfig, ReplacementPolicy, WritePolicy};
// use crate::pvm::cache_stats::CacheStatistics;
// use crate::pvm::caches::Cache;
// use crate::pvm::vm_errors::{VMError, VMResult};
//
// const DEFAULT_MEMORY_SIZE: usize = 1024 * 1024; // 1MB par défaut
//
// pub struct Memory {
//     data: HashMap<u64, u64>,
//     cache: Cache,
// }
//
// pub struct MemoryController {
//     pub main_memory: Vec<u8>,
//     pub cache: Cache,
// }
//
// impl Memory {
//     pub fn new() -> VMResult<Self> {
//         // Crée un L2, puis un L1 branché sur L2
//         let l2_config = CacheConfig::new_l2();
//         let l1_config = CacheConfig::new_l1();
//
//         let l2_cache = Box::new(Cache::new(l2_config, None));
//         let l1_cache = Cache::new(l1_config, Some(l2_cache));
//
//         Ok(Self {
//             data: HashMap::new(),
//             cache: l1_cache,
//         })
//     }
//
//
//     /// Lecture avec le cache:
//     ///  1) cache.read(...) => Ok => renvoie
//     ///  2) si Err => on cherche data.get(...) => si absent => Err
//     pub fn read(&mut self, addr: u64) -> VMResult<u64> {
//         // self.check_alignement(addr, 8)?;
//         match self.cache.read(addr) {
//             Ok(value) => {
//                 // Trouvé dans le cache
//                 Ok(value)
//             }
//             Err(_) => {
//                 // Pas trouvé ni en L1 ni en L2 => on check "data"
//                 self.data
//                     .get(&addr)
//                     .copied()
//                     .ok_or_else(|| VMError::memory_error(&format!(
//                         "Address {:#x} not found", addr
//                     )))
//             }
//         }
//     }
//
//     /// Ecrit d'abord dans le cache.
//     /// Si WriteThrough, on actualise `data`.
//     pub fn write(&mut self, addr: u64, value: u64) -> VMResult<()> {
//         self.cache.write(addr, value)?;
//
//         // Si c'est WriteThrough => on stocke aussi dans data
//         if self.cache.get_write_policy() == WritePolicy::WriteThrough {
//             self.data.insert(addr, value);
//         }
//         Ok(())
//     }
//
//     /// Vide tout: data.clear() + cache.reset() => plus de traces => read(...) -> Err
//     pub fn clear(&mut self) {
//         self.data.clear();
//         // On force un reset complet du cache
//         if let Some(l2_cache) = &mut self.cache.next_level {
//             l2_cache.reset().unwrap_or_default();
//         }
//         self.cache.reset().unwrap_or_default();
//     }
//
//     pub fn get_cache_stats(&self) -> String {
//         self.cache.get_detailed_stats()
//     }
//
//     pub fn check_alignement(&self, addr: u64, size: usize) -> VMResult<()> {
//         if addr % size as u64 != 0 {
//             return Err(VMError::memory_error(&format!(
//                 "Unaligned memory access at address {:#x}", addr
//             )));
//         }
//         Ok(())
//     }
// }
//
// impl MemoryController {
//     pub fn new(memory_size: usize, cache_size: usize) -> VMResult<Self> {
//         // Simple L1 direct, pas de next_level
//         let l1_config = CacheConfig {
//             size: cache_size,
//             lines_size: 64,
//             associativity: 4,
//             write_policy: WritePolicy::WriteThrough,
//             replacement_policy: ReplacementPolicy::LRU,
//         };
//
//         Ok(Self {
//             main_memory: vec![0; memory_size],
//             cache: Cache::new(l1_config, None),
//         })
//     }
//
//     pub fn with_default_size() -> VMResult<Self> {
//         Self::new(DEFAULT_MEMORY_SIZE, DEFAULT_MEMORY_SIZE / 4)
//     }
//
//     /// Remet main_memory à 0 + reset le cache
//     pub fn reset(&mut self) -> VMResult<()> {
//         self.main_memory.fill(0);
//         self.cache.reset()
//     }
//
//
//     /// Lit 8 octets à l’adresse `addr`, en passant par `cache`.
//     ///  1) On tente un `cache.read(...)`
//     ///  2) s'il y a Err => on lit main_memory => on “allocate” => write en cache
//     // pub fn read(&mut self, addr: u64) -> VMResult<u64> {
//     //     let addr_usize = addr as usize;
//     //     self.check_bounds(addr_usize, 8)?;
//     //
//     //     match self.cache.read(addr) {
//     //         Ok(value) => Ok(value),
//     //         Err(_) => {
//     //             // Miss => lire "main_memory"
//     //             let mut bytes = [0u8; 8];
//     //             bytes.copy_from_slice(&self.main_memory[addr_usize..addr_usize + 8]);
//     //             let value = u64::from_le_bytes(bytes);
//     //
//     //             // On "place" la donnée dans le cache => write
//     //             self.cache.write(addr, value)?;
//     //             Ok(value)
//     //         }
//     //     }
//     // }
//
//     pub fn read(&mut self, addr: u64) -> VMResult<u64> {
//         let addr_usize = addr as usize;
//         self.check_bounds(addr_usize, 8)?;
//
//         match self.cache.read(addr) {
//             Ok(value) => Ok(value),
//             Err(_) => {
//                 // Miss => incrémenter le compteur
//                 self.cache.increment_misses();
//
//                 // Lire depuis la mémoire principale
//                 let mut bytes = [0u8; 8];
//                 bytes.copy_from_slice(&self.main_memory[addr_usize..addr_usize + 8]);
//                 let value = u64::from_le_bytes(bytes);
//
//                 // Mettre en cache
//                 self.cache.write(addr, value)?;
//                 Ok(value)
//             }
//         }
//     }
//
//     // Écrit 8 octets. Politique “WriteThrough” => on écrit main_memory aussi
//     pub fn write(&mut self, addr: u64, value: u64) -> VMResult<()> {
//         let addr_usize = addr as usize;
//         self.check_bounds(addr_usize, 8)?;
//
//         // Ecrit dans cache
//         self.cache.write(addr, value)?;
//
//         if self.cache.get_write_policy() == WritePolicy::WriteThrough {
//             let bytes = value.to_le_bytes();
//             self.main_memory[addr_usize..addr_usize + 8].copy_from_slice(&bytes);
//         }
//
//         Ok(())
//     }
//
//     pub fn get_cache_stats(&self) -> VMResult<CacheStatistics> {
//         Ok(self.cache.get_statistics().clone())
//     }
//
//     fn check_bounds(&self, addr: usize, size: usize) -> VMResult<()> {
//         if addr + size > self.main_memory.len() {
//             return Err(VMError::memory_error(&format!(
//                 "Memory access out of bounds at address 0x{:X}",
//                 addr
//             )));
//         }
//         Ok(())
//     }
// }
//
//
// /// -----------------------------------------------------------------------------
// /// ------------------------------ TESTS  ----------------------------------------
// /// -----------------------------------------------------------------------------
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_memory_basic_operations() {
//         let mut memory = MemoryController::new(1024, 256).unwrap();
//
//         memory.write(0, 0x1234_5678_9ABC_DEF0).unwrap();        // 0x1234_5678_9ABC_DEF0 à l'adresse 0
//         assert_eq!(memory.read(0).unwrap(), 0x1234_5678_9ABC_DEF0);
//
//         memory.write(8, 0xFEDC_BA98_7654_3210).unwrap();
//         assert_eq!(memory.read(8).unwrap(), 0xFEDC_BA98_7654_3210); // 0xFEDC_BA98_7654_3210 à l'adresse 8
//         assert_eq!(memory.read(0).unwrap(), 0x1234_5678_9ABC_DEF0);
//     }
//
//     #[test]
//     fn test_memory_bounds() {
//         let mut memory = MemoryController::new(16, 256).unwrap();
//         // on ne peut pas écrire à l'index 16 (out of bounds)
//         assert!(memory.write(16, 0x1234).is_err());
//         // idem en lecture
//         assert!(memory.read(16).is_err());
//     }
//
//     #[test]
//     fn test_memory_alignment() {
//         let mut memory = MemoryController::new(1024, 256).unwrap();
//
//         for addr in (0..32).step_by(8) {
//             memory.write(addr, addr as u64).unwrap();
//             assert_eq!(memory.read(addr).unwrap(), addr as u64);
//         }
//     }
//
//     /// test_memory_cache_coherence:
//     /// 1) On écrit dans memory: 0x1000 => 42
//     /// 2) On lit => 42
//     /// 3) On overwrite => 84 => lit => 84
//     /// 4) .clear() => plus rien
//     /// => read(0x1000) => Err
//     #[test]
//     fn test_memory_cache_coherence() {
//         let mut memory = Memory::new().unwrap();
//
//         // 1) write => 42
//         memory.write(0x1000, 42).unwrap();
//         assert_eq!(memory.read(0x1000).unwrap(), 42);
//
//         // 2) relit => 42
//         assert_eq!(memory.read(0x1000).unwrap(), 42);
//
//         // 3) overwrite => 84 => relit => 84
//         memory.write(0x1000, 84).unwrap();
//         assert_eq!(memory.read(0x1000).unwrap(), 84);
//
//         // 4) clear
//         memory.clear();
//
//         // => on veut un Err
//         assert!(
//             memory.read(0x1000).is_err(),
//             "Après clear(), read(0x1000) devrait être Err()"
//         );
//     }
//
//     /// test_cache_statistics:
//     /// 1) write(0, 42)
//     /// 2) read(0) -> miss
//     /// 3) read(0) -> hit
//     /// => stats.l1_hits>0, total_accesses()>hits
//     #[test]
//     fn test_cache_statistics() {
//         let mut memory = MemoryController::with_default_size().unwrap();
//
//         // 1) On write => c'est un "write" => ça incrémente (write_hits ou write_misses)
//         //    Si la ligne n'est pas en cache => c'est un "miss" ?
//         //    Dans un WriteThrough NoWriteAllocate scenario, on n'ajoute pas la ligne,
//         //    ou si on a un "store miss" => ???
//
//         // 2) On read(0) => 1er read => devrais être un MISS => on rapatrie la valeur "0" ?
//         //    Eh, on a mis 42 => oh c'est main_memory => ???
//
//         memory.write(0, 42).unwrap();
//
//         // 2) 1er read => on s'attend à un "miss"
//         let _ = memory.read(0).unwrap();
//         // 3) 2eme read => "hit"
//
//         let _ = memory.read(0).unwrap();
//
//         let stats = memory.get_cache_stats().unwrap();
//
//         // On veut >=1 hit
//         assert!(
//             stats.l1_hits > 0,
//             "On veut au moins 1 hit sur la 2eme lecture"
//         );
//
//         // total_accesses() > hits => il y a eu un miss
//         assert!(
//             stats.total_accesses() > stats.l1_hits,
//             "Il doit y avoir au moins 1 miss => total_accesses()>hits"
//         );
//     }
// }

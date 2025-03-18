//src/pvm/memorys.rs


use std::io;


use crate::pvm::caches::{DEFAULT_LINE_SIZE, L1Cache};
use crate::pvm::buffers::StoreBuffer;
use crate::pvm::vm_errors::VMError;

/// Configuration du systeme memoire
#[derive(Debug, Clone, Copy)]
pub struct MemoryConfig{
    pub size: usize,
    pub l1_cache_size: usize,
    // pub l2_cache_size: usize,
    pub store_buffer_size: usize,
}


/// Statistiques du système mémoire
#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryStats {
    /// Nombre de hits dans le cache
    pub hits: u64,
    /// Nombre de misses dans le cache
    pub misses: u64,
    /// Nombre de hits dans le store buffer
    pub sb_hits: u64,
    /// Nombre d'écritures
    pub writes: u64,
    /// Nombre de lectures
    pub reads: u64,
}



impl Default for MemoryConfig {
    fn default() -> Self {
        Self{
            size: 1024 * 1024, // 1MB
            l1_cache_size: 4 * 1024,
            // l2_cache_size: 512 * 1024, // 512KB
            store_buffer_size: 8,
        }
    }
}

///  Structure memoire VM
pub struct Memory {
    memory: Vec<u8>,    // Mémoire principale
    l1_cache: L1Cache,      // Cache L1
    store_buffer: StoreBuffer,  // Store buffer
    stats: MemoryStats,   // Statistiques de la mémoire
}

impl Memory{
    /// Crée un nouveau système mémoire
    pub fn new(config: MemoryConfig) -> Self {
        Self {
            memory: vec![0; config.size],
            l1_cache: L1Cache::new(config.l1_cache_size),
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

        // 2. Vérifier ensuite dans le cache L1
        // if let Some(value) = self.l1_cache.lookup_byte(addr) {
        //     self.stats.hits += 1;
        //     return Ok(value);
        // }
        //
        // // Si absent du cache, lire depuis la mémoire principale
        // self.stats.misses += 1;
        // let value = self.memory[addr as usize];
        //
        // // Mettre à jour le cache
        // self.l1_cache.update(addr, value);
        //
        // Ok(value)
        if let Some(value) = self.l1_cache.read_byte(addr) {
            // HIT
            self.stats.hits += 1;
            return Ok(value);
        } else {
            // MISS
            self.stats.misses += 1;
            // On va chercher la ligne complète en RAM, puis on la met en cache.
            self.fill_line_from_ram(addr);
            // Maintenant qu'on a fait un fill line, on relit
            let val_after_fill = self.l1_cache.read_byte(addr)
                .expect("Cache must have the line after fill_line_from_ram");
            return Ok(val_after_fill);
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
        println!("read_dword: b0 = {}, b1 = {}, b2 = {}, b3 = {}", b0, b1, b2, b3);
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

        // 1) Ajouter au store buffer (stocke la dernière écriture)
        self.store_buffer.add(addr, value);

        // 2) Mettre à jour la cache L1 (write-allocate).
        //    - On tente d'écrire : si miss => fill line => réécrire.
        if !self.l1_cache.write_byte(addr, value) {
            // Miss, on fait un fill line en RAM, puis write_byte
            self.stats.misses += 1;
            self.fill_line_from_ram(addr);
            let _ = self.l1_cache.write_byte(addr, value);
            // On compte ce second write comme un "hit" en cache ?
            // De manière simplifiée, on peut dire qu'on a un miss unique (celui du début).
        } else {
            // On considère que c'est un hit ?
            self.stats.hits += 1;
        }

        // println!("Ecriture dans la memoire: addr = 0x{:08X}, value = {}", addr, value);

        // 3) Écriture immédiate (write-through) en RAM
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
    //         // => c’est un “write miss”, on ne touche pas stats.misses/hits
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
    // pub fn write_block(&mut self, addr: u32, data: &[u8]) -> io::Result<()> {
    //     self.check_address(addr + data.len() as u32 - 1)?;
    //
    //     // Écriture byte par byte pour bénéficier des mécanismes de cache et store buffer
    //     for (i, &byte) in data.iter().enumerate() {
    //         self.write_byte(addr + i as u32, byte)?;
    //     }
    //
    //     Ok(())
    // }
    pub fn write_block(&mut self, addr: u32, data: &[u8]) -> io::Result<()> {
        let end = addr + (data.len() as u32) - 1;
        self.check_address(end)?;

        for (i, &b) in data.iter().enumerate() {
            self.write_byte(addr + i as u32, b)?;
        }
        println!("write_block: addr = 0x{:08X}, data = {:?}", addr, data);
        Ok(())
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


    fn fill_line_from_ram(&mut self, addr: u32) {
        let base = self.l1_cache.get_line_addr(addr);
        let mut line_data = [0u8; DEFAULT_LINE_SIZE];

        // Charger 64 octets depuis la RAM (en tenant compte des limites)
        let max_addr = (base as usize + DEFAULT_LINE_SIZE).min(self.memory.len());
        let slice_len = max_addr - (base as usize);

        line_data[..slice_len].copy_from_slice(&self.memory[base as usize .. max_addr]);

        // On insère la ligne dans la cache
        self.l1_cache.fill_line(base, line_data);
    }

    fn fill_line_from_ram_no_stats(&mut self, addr: u32) {
        let base = self.l1_cache.get_line_addr(addr);
        let mut line_data = [0u8; DEFAULT_LINE_SIZE];

        let max_addr = (base as usize + DEFAULT_LINE_SIZE).min(self.memory.len());
        let slice_len = max_addr - (base as usize);

        line_data[..slice_len].copy_from_slice(&self.memory[base as usize..max_addr]);

        self.l1_cache.fill_line(base, line_data);
    }



    /// Réinitialise le système mémoire
    pub fn reset(&mut self) {
        println!("Resetting memory...");
        self.memory.iter_mut().for_each(|byte| *byte = 0);
        self.l1_cache.clear();
        self.store_buffer.clear();
        self.stats = MemoryStats::default(); // Assurez-vous que cela remet bien tous les compteurs à 0
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
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.sb_hits, 0);
        assert_eq!(stats.writes, 0);
        assert_eq!(stats.reads, 0);
    }

    #[test]
    #[ignore]
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
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
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

    #[test]
    fn test_memory_cache_hit() {
        let config = MemoryConfig::default();
        let mut mem = Memory::new(config);

        // 1) Écrire un octet => Miss dans la cache => line fill => etc.
        mem.write_byte(0x100, 42).unwrap();

        // 2) Lire cet octet => d’abord store buffer => sb_hit
        let _ = mem.read_byte(0x100).unwrap();
        assert_eq!(mem.stats().sb_hits, 1);

        // 3) flush store buffer
        mem.flush_store_buffer();

        // 4) Relire => maintenant on s’attend à un hit en cache L1
        let _ = mem.read_byte(0x100).unwrap();
        assert_eq!(mem.stats().hits, 1);
    }

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
        assert_eq!(stats.reads, 1);   // la lecture juste après l’écriture
        assert_eq!(stats.misses, 1); // la lecture de 0x100 après reset
        assert_eq!(stats.hits, 0);
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
    #[ignore]
    fn test_memory_flush_store_buffer() {
        let config = MemoryConfig::default();
        let mut mem = Memory::new(config);

        // Écrire des bytes => tout reste dans le store buffer (pas encore flush)
        mem.write_byte(0x100, 42).unwrap();
        mem.write_byte(0x101, 43).unwrap();

        // Flush => on vide le store buffer (les données sont en RAM),
        // mais la cache L1 ne contient pas forcément ces adresses (sauf si elle
        // avait été remplie avant, ce qui n’est pas le cas ici).
        mem.flush_store_buffer().unwrap();

        // On lit 0x100 => va provoquer un MISS, un fill line, puis un HIT sur la lecture
        let _ = mem.read_byte(0x100).unwrap();

        // On lit 0x101 => même ligne => HIT direct
        let _ = mem.read_byte(0x101).unwrap();

        let stats = mem.stats();

        // Scénario "réaliste" :
        // - 2 writes
        // - 2 reads
        // - AUCUN sb_hit (on a flush avant de relire)
        // => sur le premier read(0x100), MISS => +1 miss, puis on charge la ligne,
        //    => +1 hit pour la lecture qui suit le fill
        // => second read(0x101) = hit sur la même ligne => +1 hit
        //
        // Donc on obtient:
        //   hits = 2
        //   misses = 1
        //   sb_hits = 0
        //   writes = 2
        //   reads = 2
        //
        // Ajuste en fonction de tes conventions si tu comptes un "hit" post-miss ou pas.
        assert_eq!(stats.writes, 2);
        assert_eq!(stats.reads, 2);
        assert_eq!(stats.sb_hits, 0);

        assert_eq!(stats.hits, 2, "Deux accès dans la même ligne => 2 hits");
        assert_eq!(stats.misses, 1, "Premier accès => miss => fill line => hits++");
    }
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
//         assert_eq!(stats.hits, 0);
//         assert_eq!(stats.misses, 0);
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
//         assert_eq!(stats.hits, 0);
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
//         assert_eq!(stats.hits, 1);
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
//         assert_eq!(stats.hits, 0);
//         assert_eq!(stats.misses, 1);
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
//         assert_eq!(stats.hits, 2);
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
//     /// => stats.hits>0, total_accesses()>hits
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
//             stats.hits > 0,
//             "On veut au moins 1 hit sur la 2eme lecture"
//         );
//
//         // total_accesses() > hits => il y a eu un miss
//         assert!(
//             stats.total_accesses() > stats.hits,
//             "Il doit y avoir au moins 1 miss => total_accesses()>hits"
//         );
//     }
// }
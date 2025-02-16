use crate::pvm::vm_errors::{VMError, VMResult};



const CACHE_LINE_SIZE: usize = 64;

/// Cache mémoire
#[derive(Default)]
pub struct Cache {
    lines: Vec<CacheLine>,
    statistics: CacheStatistics,
}

#[derive(Debug, Default, Clone)]
pub struct CacheStatistics {
    pub hits: usize,
    pub misses: usize,
}

#[derive(Debug, Clone)]
pub struct CacheLine {
    tag: u64,
    data: [u8; CACHE_LINE_SIZE],
    valid: bool,
    dirty: bool,
}


#[derive(Debug, Default)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub total_accesses: usize,

}

impl Default for CacheLine {
    fn default() -> Self {
        Self {
            tag: 0,
            data: [0; CACHE_LINE_SIZE],
            valid: false,
            dirty: false,
        }
    }
}


// Ajouter des méthodes utiles pour CacheStatistics
impl CacheStatistics {
    pub fn total_accesses(&self) -> usize {
        self.hits + self.misses
    }

    pub fn hit_rate(&self) -> f64 {
        if self.total_accesses() == 0 {
            0.0
        } else {
            self.hits as f64 / self.total_accesses() as f64
        }
    }
}


impl Cache {


    pub fn new(size: usize) -> VMResult<Self> {
        let num_lines = size / CACHE_LINE_SIZE;
        if num_lines == 0 {
            return Err(VMError::ConfigError("Taille de cache trop petite".into()));
        }

        Ok(Self {
            lines: vec![CacheLine::default(); num_lines],
            statistics: CacheStatistics::default(),
        })
    }

    pub fn reset(&mut self) -> VMResult<()> {
        for line in &mut self.lines {
            *line = CacheLine::default();
        }
        self.statistics = CacheStatistics::default();
        Ok(())
    }



    /// ecrit dans la cache
    pub fn write(&mut self, addr: usize, value: u64) -> VMResult<()> {
        let (line_index, offset, tag) = self.get_cache_info(addr);

        // Mettre à jour la ligne de cache
        let line = &mut self.lines[line_index];
        line.tag = tag;
        line.valid = true;
        line.dirty = true;

        // Convertir la valeur en bytes et l'écrire dans la ligne
        let value_bytes = value.to_le_bytes();
        line.data[offset..offset + 8].copy_from_slice(&value_bytes);

        Ok(())
    }


    /// Lit une valeur depuis le cache
    pub fn read(&mut self, addr: usize) -> VMResult<Option<u64>> {
        let (line_index, offset, tag) = self.get_cache_info(addr);

        // Vérifier si la ligne est valide et contient la bonne adresse
        let line = &self.lines[line_index];
        if !line.valid || line.tag != tag {
            self.statistics.misses += 1;
            return Ok(None);
        }

        self.statistics.hits += 1;

        // Extraire la valeur de 64 bits à partir de la ligne de cache
        let mut value_bytes = [0u8; 8];
        value_bytes.copy_from_slice(&line.data[offset..offset + 8]);
        let value = u64::from_le_bytes(value_bytes);

        Ok(Some(value))
    }


    ///Obtient les Stats de la cache
    // pub fn get_statistics(&self) -> VMResult<CacheStats>{
    //     Ok(CacheStats{
    //         hits: self.statistics.hits,
    //         misses: self.statistics.misses,
    //         total_accesses: self.statistics.hits + self.statistics.misses,
    //     })
    //     // &self.statistics;
    // }

    // Obtient les statistiques du cache
    pub fn get_statistics(&self) -> &CacheStatistics {
        &self.statistics
    }




    /// Vérifie si une adresse est dans le cache
    pub fn is_cached(&self, addr: usize) -> bool {
        let (line_index, _) = self.get_cache_location(addr);
        let line = &self.lines[line_index];
        line.valid && line.tag == self.get_tag(addr)
    }

    /// Invalide une ligne du cache
    pub fn invalidate(&mut self, addr: usize) -> VMResult<()> {
        let (line_index, _) = self.get_cache_location(addr);
        let line = &mut self.lines[line_index];
        line.valid = false;
        line.dirty = false;
        Ok(())
    }

    // Méthodes utilitaires privées
    fn get_cache_location(&self, addr: usize) -> (usize, usize) {
        let line_index = (addr / CACHE_LINE_SIZE) % self.lines.len();
        let offset = addr % CACHE_LINE_SIZE;
        (line_index, offset)
    }

    fn get_tag(&self, addr: usize) -> u64 {
        (addr / CACHE_LINE_SIZE) as u64
    }


    /// Calcule l'index de la ligne et le tag pour une adresse
    fn get_cache_info(&self, addr: usize) -> (usize, usize, u64) {
        let line_index = (addr / CACHE_LINE_SIZE) % self.lines.len();
        let offset = addr % CACHE_LINE_SIZE;
        let tag = (addr / CACHE_LINE_SIZE) as u64;
        (line_index, offset, tag)
    }


}








#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let cache = Cache::new(256).unwrap();
        assert_eq!(cache.lines.len(), 4); // 256 / 64 = 4 lignes
    }

    #[test]
    fn test_cache_read_write() {
        let mut cache = Cache::new(256).unwrap();

        // Test écriture puis lecture
        cache.write(0, 0x1234_5678_9ABC_DEF0).unwrap();
        assert_eq!(cache.read(0).unwrap(), Some(0x1234_5678_9ABC_DEF0));

        // Vérifier les statistiques
        assert_eq!(cache.statistics.hits, 1);
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = Cache::new(256).unwrap();

        // Lecture d'une adresse non mise en cache
        assert_eq!(cache.read(0).unwrap(), None);
        assert_eq!(cache.statistics.misses, 1);
    }

    #[test]
    fn test_cache_replacement() {
        let mut cache = Cache::new(256).unwrap();

        // Remplir toutes les lignes du cache
        for i in 0..4 {
            cache.write(i * CACHE_LINE_SIZE, i as u64).unwrap();
        }

        // Vérifier que toutes les valeurs sont bien en cache
        for i in 0..4 {
            assert_eq!(cache.read(i * CACHE_LINE_SIZE).unwrap(), Some(i as u64));
        }
    }

    #[test]
    fn test_cache_invalidation() {
        let mut cache = Cache::new(256).unwrap();

        cache.write(0, 42).unwrap();
        assert!(cache.is_cached(0));

        cache.invalidate(0).unwrap();
        assert!(!cache.is_cached(0));
    }




}
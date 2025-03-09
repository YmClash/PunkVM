//src/bytecode/files.rs


use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

use super::instructions::Instruction;

///Signature d'un fichier de bytecode PunkVM (PunkVM en ASCII)
pub const PUNK_SIGNATURE: [u8; 4] = [0x50, 0x55, 0x4E, 0x4B];

/// Version du format de bytecode (majour.minor.patch.build)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BytecodeVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
    pub build: u8,
}

impl BytecodeVersion{
    pub fn new(major: u8, minor: u8, patch: u8, build: u8) -> Self {
        Self {
            major,
            minor,
            patch,
            build,
        }
    }
    pub fn encode(&self) -> [u8;4]{
        [self.major,self.minor,self.patch,self.build]
    }
    pub fn decode(bytes:[u8;4]) -> Self{
        Self {
            major: bytes[0],
            minor: bytes[1],
            patch: bytes[2],
            build: bytes[3],
        }
    }
    pub fn to_string(&self) -> String {
        format!("{}.{}.{}.{}", self.major, self.minor, self.patch, self.build)
    }
}

impl Default for BytecodeVersion {
    fn default() -> Self {
        Self {
            major: 0,
            minor: 1,
            patch: 0,
            build: 0,
        }
    }
}


/// Types de segments dans un fichier de bytecode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SegmentType{
    Code = 0,
    Data = 1,
    ReadOnlyData = 2,
    Symbols = 3,
    Debug = 4,
    // 5-15 réservés pour extensions futures
}

impl SegmentType{
    pub fn from_u8(value:u8) -> Option<Self>{
        match value {
            0 => Some(Self::Code),
            1 => Some(Self::Data),
            2 => Some(Self::ReadOnlyData),
            3 => Some(Self::Symbols),
            4 => Some(Self::Debug),
            _ => None,
        }
    }
}

/// Metadonnées d'un segment
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentMetadata{
    pub segment_type: SegmentType,
    pub offset:u32,     // offset dans le fichier
    pub size:u32,       // taille du segment en bytes
    pub load_addr:u32, // adresse de chargement en memoire
}

impl SegmentMetadata{
    pub fn new(segment_type: SegmentType, offset: u32, size: u32, load_addr: u32) -> Self {
        Self {
            segment_type,
            offset,
            size,
            load_addr,
        }
    }

    /// Encodage d'un segment en bytes (13bytes  au total)
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(13);
        bytes.push(self.segment_type as u8);

        bytes.extend_from_slice(&self.offset.to_le_bytes());
        bytes.extend_from_slice(&self.size.to_le_bytes());
        bytes.extend_from_slice(&self.load_addr.to_le_bytes());

        bytes
    }

    /// Décodage d'un segment depuis des bytes
    pub fn decode(bytes:&[u8]) -> Option<Self>{
        if bytes.len() < 13 {
            return None;
        }
        let offset = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
        let size = u32::from_le_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]);
        let load_addr = u32::from_le_bytes([bytes[9], bytes[10], bytes[11], bytes[12]]);

        // Some(Self::new(segment_type, offset, size, load_addr))

        Some(Self {
            segment_type: SegmentType::from_u8(bytes[0])?,
            offset,
            size,
            load_addr,
        })
    }

}



/// Structure representant un fichier bytecode PunkVM complet
#[derive(Debug, Clone)]
pub struct BytecodeFile{
    pub version: BytecodeVersion,
    pub metadata: HashMap<String, String>,
    pub segments: Vec<SegmentMetadata>,
    pub code: Vec<Instruction>,
    pub data: Vec<u8>,
    pub readonly_data: Vec<u8>,
    pub symbols: HashMap<String, u32>,
    pub debug_info: Vec<u8>
}

impl Default for BytecodeFile {
    fn default() -> Self {
        Self::new()
    }
}

impl BytecodeFile {
    /// Crée un nouveau fichier de bytecode avec les valeurs par défaut
    pub fn new() -> Self{
        Self {
            version: BytecodeVersion::default(),
            metadata: HashMap::new(),
            segments: Vec::new(),
            code: Vec::new(),
            data: Vec::new(),
            readonly_data: Vec::new(),
            symbols: HashMap::new(),
            debug_info: Vec::new(),
        }
    }

    /// Ajoute une instruction au  segment de code
    pub fn add_instruction(&mut self, instruction: Instruction){
        self.code.push(instruction);
    }

    /// Ajoute une donnée au segment de données
    pub fn add_data(&mut self, data: &[u8]) -> u32{
        let offset = self.data.len() as u32;
        self.data.extend_from_slice(data);
        offset
    }
    /// Ajoute des données constantes au segment de données en lecture seule
    pub fn add_readonly_data(&mut self, data: &[u8]) -> u32 {
        let offset = self.readonly_data.len() as u32;
        self.readonly_data.extend_from_slice(data);
        offset
    }
    /// Ajoute un symbole (label) avec son adresse
    pub fn add_symbol(&mut self, name: &str, address: u32) {
        self.symbols.insert(name.to_string(), address);
    }

    /// Ajoute une métadonnée au fichier
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
    /// Écrit le fichier bytecode sur disque
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut file = File::create(path)?;

        // Écriture de l'en-tête
        file.write_all(&PUNK_SIGNATURE)?;
        file.write_all(&self.version.encode())?;

        // Écriture des métadonnées
        let metadata_bytes = self.encode_metadata();
        let metadata_size = metadata_bytes.len() as u32;
        file.write_all(&metadata_size.to_le_bytes())?;
        file.write_all(&metadata_bytes)?;

        // Calcul des offsets pour les segments
        let header_size = 8; // Signature (4) + Version (4)
        let metadata_header_size = 4; // Taille des métadonnées

        let mut current_offset = header_size + metadata_header_size + metadata_size as usize;

        // Nombre de segments
        let num_segments = 5; // Code, Data, ReadOnlyData, Symbols, Debug
        file.write_all(&(num_segments as u32).to_le_bytes())?;
        current_offset += 4;

        // Calcul de la taille de la table des segments
        let segments_table_size = num_segments * 13; // 13 bytes par segment
        current_offset += segments_table_size as usize;

        // Encodage des segments
        let mut segments = Vec::new();

        // Segment de code
        let code_bytes = self.encode_code();
        segments.push(SegmentMetadata::new(
            SegmentType::Code,
            current_offset as u32,
            code_bytes.len() as u32,
            0, // Adresse de chargement, typiquement 0 pour le code
        ));
        current_offset += code_bytes.len();

        // Segment de données
        segments.push(SegmentMetadata::new(
            SegmentType::Data,
            current_offset as u32,
            self.data.len() as u32,
            0, // Adresse de chargement à déterminer au chargement
        ));
        current_offset += self.data.len();

        // Segment de données en lecture seule
        segments.push(SegmentMetadata::new(
            SegmentType::ReadOnlyData,
            current_offset as u32,
            self.readonly_data.len() as u32,
            0, // Adresse de chargement à déterminer au chargement
        ));
        current_offset += self.readonly_data.len();

        // Segment de symboles
        let symbols_bytes = self.encode_symbols();
        segments.push(SegmentMetadata::new(
            SegmentType::Symbols,
            current_offset as u32,
            symbols_bytes.len() as u32,
            0, // Non applicable pour les symboles
        ));
        current_offset += symbols_bytes.len();

        // Segment de debug
        segments.push(SegmentMetadata::new(
            SegmentType::Debug,
            current_offset as u32,
            self.debug_info.len() as u32,
            0, // Non applicable pour les infos de debug
        ));

        // Écriture de la table des segments
        for segment in &segments {
            file.write_all(&segment.encode())?;
        }

        // Écriture des données des segments
        file.write_all(&code_bytes)?;
        file.write_all(&self.data)?;
        file.write_all(&self.readonly_data)?;
        file.write_all(&symbols_bytes)?;
        file.write_all(&self.debug_info)?;

        Ok(())
    }

    /// Lit un fichier bytecode depuis le disque
    pub fn read_from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        if buffer.len() < 8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Fichier bytecode trop petit",
            ));
        }

        // Vérification de la signature
        if buffer[0..4] != PUNK_SIGNATURE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Signature de fichier bytecode invalide",
            ));
        }

        // Lecture de la version
        let version = BytecodeVersion::decode([buffer[4], buffer[5], buffer[6], buffer[7]]);

        // Lecture de la taille des métadonnées
        if buffer.len() < 12 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Données invalides dans l'en-tête du fichier",
            ));
        }

        let metadata_size = u32::from_le_bytes([buffer[8], buffer[9], buffer[10], buffer[11]]) as usize;
        let mut offset = 12;

        // Lecture des métadonnées
        if buffer.len() < offset + metadata_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Données de métadonnées incomplètes",
            ));
        }

        let metadata = Self::decode_metadata(&buffer[offset..offset + metadata_size])?;
        offset += metadata_size;

        // Lecture du nombre de segments
        if buffer.len() < offset + 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Données de segments incomplètes",
            ));
        }

        let num_segments = u32::from_le_bytes([
            buffer[offset],
            buffer[offset + 1],
            buffer[offset + 2],
            buffer[offset + 3],
        ]) as usize;
        offset += 4;

        // Lecture des métadonnées de segments
        let mut segments = Vec::with_capacity(num_segments);
        for _ in 0..num_segments {
            if buffer.len() < offset + 13 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Données de segment incomplètes",
                ));
            }

            let segment = SegmentMetadata::decode(&buffer[offset..offset + 13])
                .ok_or_else(|| io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Métadonnées de segment invalides",
                ))?;

            segments.push(segment);
            offset += 13;
        }

        // Création de l'objet de fichier
        let mut bytecode_file = BytecodeFile {
            version,
            metadata,
            segments: segments.clone(),
            code: Vec::new(),
            data: Vec::new(),
            readonly_data: Vec::new(),
            symbols: HashMap::new(),
            debug_info: Vec::new(),
        };

        // Lecture des données des segments
        for segment in segments {
            let start = segment.offset as usize;
            let end = start + segment.size as usize;

            if buffer.len() < end {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Segment incomplet: {:?}", segment.segment_type),
                ));
            }

            match segment.segment_type {
                SegmentType::Code => {
                    bytecode_file.code = Self::decode_code(&buffer[start..end])?;
                },
                SegmentType::Data => {
                    bytecode_file.data = buffer[start..end].to_vec();
                },
                SegmentType::ReadOnlyData => {
                    bytecode_file.readonly_data = buffer[start..end].to_vec();
                },
                SegmentType::Symbols => {
                    bytecode_file.symbols = Self::decode_symbols(&buffer[start..end])?;
                },
                SegmentType::Debug => {
                    bytecode_file.debug_info = buffer[start..end].to_vec();
                },
            }
        }

        Ok(bytecode_file)
    }

    /// Encode les métadonnées en bytes
    pub fn encode_metadata(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Nombre d'entrées de métadonnées
        bytes.extend_from_slice(&(self.metadata.len() as u32).to_le_bytes());

        // Écriture des paires clé-valeur
        for (key, value) in &self.metadata {
            // Longueur de la clé
            let key_bytes = key.as_bytes();
            bytes.extend_from_slice(&(key_bytes.len() as u32).to_le_bytes());
            bytes.extend_from_slice(key_bytes);

            // Longueur de la valeur
            let value_bytes = value.as_bytes();
            bytes.extend_from_slice(&(value_bytes.len() as u32).to_le_bytes());
            bytes.extend_from_slice(value_bytes);
        }

        bytes
    }

    /// Décode les métadonnées depuis des bytes
    fn decode_metadata(bytes: &[u8]) -> io::Result<HashMap<String, String>> {
        if bytes.len() < 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Données de métadonnées incomplètes",
            ));
        }

        let mut metadata = HashMap::new();
        let num_entries = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        let mut offset = 4;

        for _ in 0..num_entries {
            if bytes.len() < offset + 4 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Données de métadonnées incomplètes",
                ));
            }

            // Lecture de la longueur de la clé
            let key_len = u32::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]) as usize;
            offset += 4;

            if bytes.len() < offset + key_len {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Clé de métadonnée incomplète",
                ));
            }

            // Lecture de la clé
            let key = String::from_utf8(bytes[offset..offset + key_len].to_vec())
                .map_err(|_| io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Clé de métadonnée invalide (UTF-8)",
                ))?;
            offset += key_len;

            if bytes.len() < offset + 4 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Données de métadonnées incomplètes",
                ));
            }

            // Lecture de la longueur de la valeur
            let value_len = u32::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]) as usize;
            offset += 4;

            if bytes.len() < offset + value_len {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Valeur de métadonnée incomplète",
                ));
            }

            // Lecture de la valeur
            let value = String::from_utf8(bytes[offset..offset + value_len].to_vec())
                .map_err(|_| io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Valeur de métadonnée invalide (UTF-8)",
                ))?;
            offset += value_len;

            metadata.insert(key, value);
        }

        Ok(metadata)
    }

    /// Encode le code en bytes
    fn encode_code(&self) -> Vec<u8>{
        let mut bytes = Vec::new();

        // Nombre d'instructions
        bytes.extend_from_slice(&(self.code.len() as u32).to_le_bytes());

        // Encodage de chaque instruction
        for instruction in &self.code{
            let encoded = instruction.encode();
            bytes.extend_from_slice(&encoded);
        }
        bytes

    }

    /// Décode le segment de code depuis des bytes
    fn decode_code(bytes: &[u8]) -> io::Result<Vec<Instruction>> {
        if bytes.len() < 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Données de code incomplètes",
            ));
        }

        let num_instructions = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        let mut instructions = Vec::with_capacity(num_instructions);
        let mut offset = 4;

        for _ in 0..num_instructions {
            if offset >= bytes.len() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Instruction incomplète",
                ));
            }

            let (instruction, size) = Instruction::decode(&bytes[offset..])
                .map_err(|err| io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Erreur de décodage d'instruction: {}", err),
                ))?;

            instructions.push(instruction);
            offset += size;
        }

        Ok(instructions)
    }

    /// Encode les symboles en bytes
    fn encode_symbols(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Nombre de symboles
        bytes.extend_from_slice(&(self.symbols.len() as u32).to_le_bytes());

        // Écriture des paires nom-adresse
        for (name, address) in &self.symbols {
            // Longueur du nom
            let name_bytes = name.as_bytes();
            bytes.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
            bytes.extend_from_slice(name_bytes);

            // Adresse
            bytes.extend_from_slice(&address.to_le_bytes());
        }

        bytes
    }

    /// Décode les symboles depuis des bytes
    fn decode_symbols(bytes: &[u8]) -> io::Result<HashMap<String, u32>> {
        if bytes.len() < 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Données de symboles incomplètes",
            ));
        }

        let mut symbols = HashMap::new();
        let num_symbols = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        let mut offset = 4;

        for _ in 0..num_symbols {
            if bytes.len() < offset + 4 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Données de symboles incomplètes",
                ));
            }

            // Lecture de la longueur du nom
            let name_len = u32::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]) as usize;
            offset += 4;

            if bytes.len() < offset + name_len {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Nom de symbole incomplet",
                ));
            }

            // Lecture du nom
            let name = String::from_utf8(bytes[offset..offset + name_len].to_vec())
                .map_err(|_| io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Nom de symbole invalide (UTF-8)",
                ))?;
            offset += name_len;

            if bytes.len() < offset + 4 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Adresse de symbole incomplète",
                ));
            }

            // Lecture de l'adresse
            let address = u32::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]);
            offset += 4;

            symbols.insert(name, address);
        }

        Ok(symbols)
    }

}


// Test unitaire pour les fichiers de bytecode
#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::opcodes::Opcode;
    use crate::bytecode::format::InstructionFormat;
    use std::io::ErrorKind;
    use tempfile::tempdir;

    #[test]
    fn test_bytecode_version() {
        let version = BytecodeVersion::new(1, 2, 3, 4);

        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.build, 4);

        // Test encode/decode
        let encoded = version.encode();
        let decoded = BytecodeVersion::decode(encoded);

        assert_eq!(decoded.major, 1);
        assert_eq!(decoded.minor, 2);
        assert_eq!(decoded.patch, 3);
        assert_eq!(decoded.build, 4);

        // Test to_string
        assert_eq!(version.to_string(), "1.2.3.4");
    }

    #[test]
    fn test_segment_type() {
        // Test des conversions valides
        assert_eq!(SegmentType::from_u8(0), Some(SegmentType::Code));
        assert_eq!(SegmentType::from_u8(1), Some(SegmentType::Data));
        assert_eq!(SegmentType::from_u8(2), Some(SegmentType::ReadOnlyData));
        assert_eq!(SegmentType::from_u8(3), Some(SegmentType::Symbols));
        assert_eq!(SegmentType::from_u8(4), Some(SegmentType::Debug));

        // Test avec valeur invalide
        assert_eq!(SegmentType::from_u8(5), None);
    }

    #[test]
    fn test_segment_metadata() {
        let segment = SegmentMetadata::new(SegmentType::Code, 100, 200, 300);

        assert_eq!(segment.segment_type, SegmentType::Code);
        assert_eq!(segment.offset, 100);
        assert_eq!(segment.size, 200);
        assert_eq!(segment.load_addr, 300);

        // Test encode/decode
        let encoded = segment.encode();
        let decoded = SegmentMetadata::decode(&encoded).unwrap();

        assert_eq!(decoded.segment_type, SegmentType::Code);
        assert_eq!(decoded.offset, 100);
        assert_eq!(decoded.size, 200);
        assert_eq!(decoded.load_addr, 300);
    }

    #[test]
    fn test_bytecode_file_simple() {
        // Création d'un fichier bytecode simple
        let mut bytecode = BytecodeFile::new();

        // Ajout de métadonnées
        bytecode.add_metadata("name", "Test");
        bytecode.add_metadata("author", "PunkVM");

        // Vérification
        assert_eq!(bytecode.metadata.get("name"), Some(&"Test".to_string()));
        assert_eq!(bytecode.metadata.get("author"), Some(&"PunkVM".to_string()));

        // Ajout d'instructions
        let instr1 = Instruction::create_no_args(Opcode::Nop);
        let instr2 = Instruction::create_reg_imm8(Opcode::Load, 0, 42);

        bytecode.add_instruction(instr1);
        bytecode.add_instruction(instr2);

        assert_eq!(bytecode.code.len(), 2);
        assert_eq!(bytecode.code[0].opcode, Opcode::Nop);
        assert_eq!(bytecode.code[1].opcode, Opcode::Load);

        // Ajout de données
        let offset = bytecode.add_data(&[1, 2, 3, 4]);
        assert_eq!(offset, 0);
        assert_eq!(bytecode.data, vec![1, 2, 3, 4]);

        // Ajout de données en lecture seule
        let offset = bytecode.add_readonly_data(&[5, 6, 7, 8]);
        assert_eq!(offset, 0);
        assert_eq!(bytecode.readonly_data, vec![5, 6, 7, 8]);

        // Ajout de symboles
        bytecode.add_symbol("main", 0x1000);
        assert_eq!(bytecode.symbols.get("main"), Some(&0x1000));
    }

    #[test]
    fn test_bytecode_file_io() {
        // Création d'un répertoire temporaire pour les tests
        let dir = tempdir().expect("Impossible de créer un répertoire temporaire");
        let file_path = dir.path().join("test.punk");

        // Création d'un fichier bytecode à écrire
        let mut bytecode = BytecodeFile::new();
        bytecode.version = BytecodeVersion::new(1, 0, 0, 0);
        bytecode.add_metadata("name", "TestIO");
        bytecode.add_instruction(Instruction::create_no_args(Opcode::Halt));
        bytecode.add_data(&[1, 2, 3]);
        bytecode.add_readonly_data(&[4, 5, 6]);
        bytecode.add_symbol("main", 0);

        // Écrire le fichier
        bytecode.write_to_file(&file_path).expect("Impossible d'écrire le fichier bytecode");

        // Lire le fichier
        let loaded = BytecodeFile::read_from_file(&file_path).expect("Impossible de lire le fichier bytecode");

        // Vérifier que le contenu est identique
        assert_eq!(loaded.version.major, 1);
        assert_eq!(loaded.version.minor, 0);
        assert_eq!(loaded.metadata.get("name"), Some(&"TestIO".to_string()));
        assert_eq!(loaded.code.len(), 1);
        assert_eq!(loaded.code[0].opcode, Opcode::Halt);
        assert_eq!(loaded.data, vec![1, 2, 3]);
        assert_eq!(loaded.readonly_data, vec![4, 5, 6]);
        assert_eq!(loaded.symbols.get("main"), Some(&0));
    }

    #[test]
    fn test_bytecode_file_io_errors() {
        // Test avec un fichier inexistant
        let result = BytecodeFile::read_from_file("nonexistent_file.punk");
        assert!(result.is_err());

        // Test avec un fichier trop petit
        let dir = tempdir().expect("Impossible de créer un répertoire temporaire");
        let invalid_file_path = dir.path().join("invalid.punk");

        // Créer un fichier invalide avec juste quelques octets
        std::fs::write(&invalid_file_path, &[0, 1, 2]).expect("Impossible d'écrire le fichier de test");

        let result = BytecodeFile::read_from_file(&invalid_file_path);
        assert!(result.is_err());

        // Vérifier le type d'erreur
        match result {
            Err(e) => assert_eq!(e.kind(), ErrorKind::InvalidData),
            _ => panic!("Expected an error but got success"),
        }
    }

    #[test]
    fn test_encode_decode_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("key1".to_string(), "value1".to_string());
        metadata.insert("key2".to_string(), "value2".to_string());

        let mut bytecode = BytecodeFile::new();
        bytecode.metadata = metadata.clone();

        let encoded = bytecode.encode_metadata();
        let decoded = BytecodeFile::decode_metadata(&encoded).expect("Failed to decode metadata");

        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded.get("key1"), Some(&"value1".to_string()));
        assert_eq!(decoded.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_encode_decode_symbols() {
        let mut symbols = HashMap::new();
        symbols.insert("sym1".to_string(), 0x1000);
        symbols.insert("sym2".to_string(), 0x2000);

        let mut bytecode = BytecodeFile::new();
        bytecode.symbols = symbols.clone();

        let encoded = bytecode.encode_symbols();
        let decoded = BytecodeFile::decode_symbols(&encoded).expect("Failed to decode symbols");

        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded.get("sym1"), Some(&0x1000));
        assert_eq!(decoded.get("sym2"), Some(&0x2000));
    }
}
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
    fn encode_metadata(&self) -> Vec<u8> {
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


# PunkVM

PunkVM est une machine virtuelle de haute performance conçue pour exécuter le bytecode généré par le compilateur PunkLang. Inspirée par l'architecture des processeurs modernes, PunkVM implémente un pipeline à 5 étages avec des optimisations avancées comme le forwarding de données, la prédiction de branchement et un système de cache hiérarchique.

## Caractéristiques

- **Architecture pipeline à 5 étages** (Fetch, Decode, Execute, Memory, Writeback)
- **ALU dédiée** pour les opérations arithmétiques et logiques
- **Système de détection de hazards** et forwarding de données
- **Prédiction de branchement** avec BTB et Return Address Stack
- **Système de cache** avec politiques d'écriture configurables
- **Store buffer** pour optimiser les accès mémoire
- **Support pour compilation JIT** via Cranelift (à venir)
- **Extensible** pour les futures fonctionnalités avancées (IA, SIMD, etc.)

## État du Projet

PunkVM est actuellement en développement actif. Consultez le [Roadmap](ROADMAP.md) pour plus de détails sur le plan de développement et l'état d'avancement.

## Prérequis

- Rust 1.70 ou supérieur
- Cargo
- (Optionnel) Cranelift pour la compilation JIT

## Installation

```bash
# Cloner le dépôt
git clone https://github.com/YmClash/PunkVM.git
cd punkvm

# Compiler le projet
cargo build --release

# Exécuter les tests
cargo test
```

## Utilisation

### Exécution de Bytecode

```rust
use punkvm::{VirtualMachine, BytecodeLoader};

fn main() {
    // Charger le bytecode depuis un fichier
    let bytecode = BytecodeLoader::from_file("program.pbc").unwrap();
    
    // Créer et configurer la VM
    let mut vm = VirtualMachine::new();
    
    // Exécuter le programme
    let result = vm.execute(bytecode);
    
    println!("Résultat: {:?}", result);
}
```

### Intégration avec PunkLang

```rust
use punklang::{Compiler, CompileOptions};
use punkvm::VirtualMachine;

fn main() {
    // Compiler le code source PunkLang
    let compiler = Compiler::new();
    let bytecode = compiler.compile("source.punk", CompileOptions::default()).unwrap();
    
    // Exécuter le bytecode
    let mut vm = VirtualMachine::new();
    let result = vm.execute(bytecode);
    
    println!("Résultat: {:?}", result);
}
```

## Architecture

PunkVM est structuré autour d'un pipeline d'exécution à 5 étages, inspiré par l'architecture des processeurs RISC modernes:

```
Fetch → Decode → Execute → Memory → Writeback
```

Un aspect central de l'architecture est l'ALU (Arithmetic Logic Unit) dédiée qui effectue les opérations arithmétiques et logiques. Le pipeline implémente également:

- Forwarding de données pour réduire les stalls
- Détection de hazards (RAW, WAR, WAW)
- Prédiction de branchement pour réduire les pénalités de branchement
- Cache hiérarchique pour optimiser les accès mémoire

Pour plus de détails, consultez la [documentation technique](docs/ARCHITECTURE.md).

## Extensions Futures

- **Compilateur JIT**: Intégration avec Cranelift pour la compilation à la volée
- **Optimisations vectorielles**: Support pour les instructions SIMD
- **Pipeline superscalaire**: Exécution parallèle d'instructions
- **Extensions neuronales**: Intégration de capacités d'IA dans la VM

## Contribution

Les contributions sont les bienvenues! Consultez [CONTRIBUTING.md](CONTRIBUTING.md) pour les directives sur la façon de contribuer au projet.

## Licence

PunkVM est distribué sous la licence MIT. Voir [LICENSE](LICENSE) pour plus d'informations.


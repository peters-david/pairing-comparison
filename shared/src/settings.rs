use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneticAlgorithmSettings {
    population_size: usize,
    survival_rate: f64,
    generations: usize,
    mutation_rate: f64,
    mutation_strength: usize,
}

impl GeneticAlgorithmSettings {
    pub fn new(
        population_size: usize,
        survival_rate: f64,
        generations: usize,
        mutation_rate: f64,
        mutation_strength: usize,
    ) -> Self {
        assert!(population_size > 2, "Population size must be above 2");
        assert!(survival_rate > 0.0, "Survival rate must be above 0");
        assert!(survival_rate < 100.0, "Survival rate must be below 100");
        assert!(mutation_rate > 0.0, "Mutation rate must be above 0");
        assert!(mutation_rate < 100.0, "Mutation rate must be below 100");
        assert!(mutation_strength > 0, "Mutation strength must be above 0");
        Self {
            population_size,
            survival_rate,
            generations,
            mutation_rate,
            mutation_strength,
        }
    }

    pub fn population_size(&self) -> usize {
        self.population_size
    }

    pub fn survival_rate(&self) -> f64 {
        self.survival_rate
    }

    pub fn generations(&self) -> usize {
        self.generations
    }

    pub fn mutation_rate(&self) -> f64 {
        self.mutation_rate
    }

    pub fn mutation_strength(&self) -> usize {
        self.mutation_strength
    }

    pub fn description(&self) -> String {
        format!(
            "pop:{} surv:{} gens:{} mut_rate:{} mut_strength:{}",
            self.population_size,
            self.survival_rate,
            self.generations,
            self.mutation_rate,
            self.mutation_strength
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProblemSettings {
    Tsp {
        size: usize,
    },
    Rofa {
        nodes: usize,
        links_percentage: usize,
        demands_percentage: usize,
        link_types: usize,
    },
}

impl ProblemSettings {
    pub fn description(&self) -> String {
        match self {
            ProblemSettings::Tsp { size } => format!("Tsp size:{}", size),
            ProblemSettings::Rofa {
                nodes,
                links_percentage,
                demands_percentage,
                link_types,
            } => format!(
                "Rofa nodes:{} links_per:{} demands_per:{} link_types:{}",
                nodes, links_percentage, demands_percentage, link_types
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PairingSettings {
    AsexualPairing,
    TwoRandomPairing,
    OneRandomPairing,
    NeighborPairing,
    SimilarFitnessPairing,
    FitnessProportionatePairing,
    ElitePairing,
    AntiElitePairing,
}

impl PairingSettings {
    pub fn name(&self) -> String {
        match self {
            PairingSettings::AsexualPairing => "ap".to_string(),
            PairingSettings::TwoRandomPairing => "trp".to_string(),
            PairingSettings::OneRandomPairing => "orp".to_string(),
            PairingSettings::NeighborPairing => "np".to_string(),
            PairingSettings::SimilarFitnessPairing => "sfp".to_string(),
            PairingSettings::FitnessProportionatePairing => "fpp".to_string(),
            PairingSettings::ElitePairing => "ep".to_string(),
            PairingSettings::AntiElitePairing => "aep".to_string(),
        }
    }

    pub fn description(&self) -> String {
        match self {
            PairingSettings::AsexualPairing => format!("Pairing: Asexual"),
            PairingSettings::TwoRandomPairing => format!("Pairing: TwoRandom"),
            PairingSettings::OneRandomPairing => format!("Pairing: OneRandom"),
            PairingSettings::NeighborPairing => format!("Pairing: Neighbor"),
            PairingSettings::SimilarFitnessPairing => format!("Pairing: SimilarFitness"),
            PairingSettings::FitnessProportionatePairing => {
                format!("Pairing: FitnessProportionate")
            }
            PairingSettings::ElitePairing => format!("Pairing: Elite"),
            PairingSettings::AntiElitePairing => format!("Pairing: AntiElite"),
        }
    }
}

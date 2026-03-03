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
        assert!(population_size > 0, "Population size must be above 2");
        assert!(survival_rate > 0.0, "Survival rate must be above 0");
        assert!(mutation_rate > 0.0, "Mutation rate must be above 0");
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

#[derive(Copy, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IndividualQuantity {
    Random,
    Elite { percentage: usize },
    AntiElite { percentage: usize },
    FitnessProportionate,
}

impl IndividualQuantity {
    pub fn description(&self) -> String {
        match self {
            IndividualQuantity::Random => format!("Random"),
            IndividualQuantity::Elite { percentage } => format!("Elite({}%)", percentage),
            IndividualQuantity::AntiElite { percentage } => format!("AntiElite({}%)", percentage),
            IndividualQuantity::FitnessProportionate => format!("FitnessProportionate"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PairingSettings {
    AsexualPairing {
        quantity: IndividualQuantity,
    },
    RandomPairing {
        quantity: IndividualQuantity,
    },
    SimilarFitnessPairing {
        quantity: IndividualQuantity,
        similarity: usize,
    },
    SpatialDistancePairing {
        quantity: IndividualQuantity,
        desired_individual_distance_percentage: usize,
    },
}

impl PairingSettings {
    pub fn description(&self) -> String {
        match self {
            PairingSettings::AsexualPairing { quantity } => {
                format!("Asexual<{}>", quantity.description())
            }
            PairingSettings::RandomPairing { quantity } => {
                format!("Random<{}>", quantity.description())
            }
            PairingSettings::SimilarFitnessPairing {
                quantity,
                similarity,
            } => {
                format!(
                    "SimilarFitness({}%)<{}>",
                    similarity,
                    quantity.description(),
                )
            }
            PairingSettings::SpatialDistancePairing {
                quantity,
                desired_individual_distance_percentage,
            } => format!(
                "SpatialDistance({}%)<{}>",
                desired_individual_distance_percentage,
                quantity.description(),
            ),
        }
    }

    pub fn quantity(&self) -> &IndividualQuantity {
        match self {
            PairingSettings::AsexualPairing { quantity } => quantity,
            PairingSettings::RandomPairing { quantity } => quantity,
            PairingSettings::SimilarFitnessPairing {
                quantity,
                similarity,
            } => quantity,
            PairingSettings::SpatialDistancePairing {
                quantity,
                desired_individual_distance_percentage,
            } => quantity,
        }
    }
}

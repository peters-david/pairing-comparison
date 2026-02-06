use std::{any::Any, marker::PhantomData};

use rand::{seq::IndexedRandom, Rng};
use shared::{
    settings::{GeneticAlgorithmSettings, PairingSettings, ProblemSettings},
    statistics::Statistic,
};

use crate::problems::{
    rofa::{Rofa, RoutingAndCapacityPlan},
    tsp::{Cities, Sequence},
};

pub struct GeneticAlgorithm<P: Problem<Individual = I>, X: Pairing<I>, I: Individual<Problem = P>> {
    problem: P,
    pairing: X,
    genetic_algorithm_settings: GeneticAlgorithmSettings,
    statistic: Statistic,
    _marker: PhantomData<I>,
}

impl<P: Problem<Individual = I>, X: Pairing<I>, I: Individual<Problem = P>>
    GeneticAlgorithm<P, X, I>
{
    pub fn new(
        problem: P,
        pairing: X,
        genetic_algorithm_settings: GeneticAlgorithmSettings,
    ) -> Self {
        let statistic = Statistic::new(
            genetic_algorithm_settings.clone(),
            problem.problem_settings(),
            pairing.pairing_settings(),
        );
        Self {
            problem,
            pairing,
            genetic_algorithm_settings,
            statistic,
            _marker: PhantomData,
        }
    }

    pub fn run(&mut self) {
        //TODO: currently all starting individuals are the same
        //let mut individuals = (0..self.settings.population_size())
        //    .map(|_| self.problem.random_individual())
        //    .collect();
        let individual = self.problem.random_individual();
        let mut individuals = (0..self.genetic_algorithm_settings.population_size())
            .map(|_| individual.clone())
            .collect();
        for generation in 0..self.genetic_algorithm_settings.generations() {
            individuals = self.step(individuals);
        }
    }

    pub fn get_statistic(self) -> Statistic {
        self.statistic
    }

    fn step(&mut self, individuals: Vec<I>) -> Vec<I> {
        let selected_individuals_with_fitness = self.select(individuals);
        let recombined_individuals = self.recombine(selected_individuals_with_fitness);
        let mutated_individuals = self.mutate(recombined_individuals);
        mutated_individuals
    }

    fn select(&mut self, individuals: Vec<I>) -> Vec<(f64, I)> {
        let mut individuals_and_fitness: Vec<(usize, f64, I)> = individuals
            .into_iter()
            .enumerate()
            .map(|(i, individual)| (i, self.problem.fitness(&individual), individual))
            .collect(); // TODO: extra data structure not neccessary, but may be in the future
        let fitness_values = individuals_and_fitness.iter().map(|(_, f, _)| *f).collect();
        self.statistic.append_fitness_values(fitness_values);
        individuals_and_fitness.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .expect("Tried to compare invalid numbers")
        });
        assert!(individuals_and_fitness[0].1 >= individuals_and_fitness[1].1);
        let number_selection_survivors = (individuals_and_fitness.len() as f64
            * self.genetic_algorithm_settings.survival_rate())
        .ceil() as usize;
        individuals_and_fitness.truncate(number_selection_survivors);
        individuals_and_fitness.sort_by(|a, b| {
            a.0.partial_cmp(&b.0)
                .expect("Tried to compare invalid numbers")
        });
        individuals_and_fitness
            .into_iter()
            .map(|(_, f, i)| (f, i))
            .collect()
    }

    fn recombine(&mut self, individuals_with_fitness: Vec<(f64, I)>) -> Vec<I> {
        let pairs = self
            .pairing
            .pair(individuals_with_fitness, &self.genetic_algorithm_settings);
        let recombined_individuals = pairs
            .iter()
            .map(|(a, b)| I::crossover(a, b, &self.problem))
            .collect();
        recombined_individuals
    }

    fn mutate(&self, mut individuals: Vec<I>) -> Vec<I> {
        let mut rng = rand::rng();
        for individual in &mut individuals {
            if self.genetic_algorithm_settings.mutation_rate() >= rng.random_range(0.0..=1.0) {
                for _ in 0..self.genetic_algorithm_settings.mutation_strength() {
                    individual.mutate(&self.problem);
                }
            }
        }
        individuals
    }
}

pub trait Individual: Any + Clone {
    type Problem: Problem<Individual = Self>;
    fn crossover(first: &Self, second: &Self, problem: &Self::Problem) -> Self;
    fn mutate(&mut self, problem: &Self::Problem);
}

pub trait Problem {
    type Individual: Individual<Problem = Self>;
    fn random(problem_settings: &ProblemSettings) -> Self;
    fn random_individual(&self) -> Self::Individual;
    fn fitness(&self, individual: &Self::Individual) -> f64;
    fn problem_settings(&self) -> ProblemSettings;
}

#[derive(Clone)]
pub enum Problems {
    Tsp(Cities),
    Rofa(Rofa),
}

pub trait Pairing<I: Individual> {
    fn name(&self) -> String;
    fn pairing_settings(&self) -> PairingSettings;
    fn pair(
        &mut self,
        individuals_with_fitness: Vec<(f64, I)>,
        settings: &GeneticAlgorithmSettings,
    ) -> Vec<(I, I)>;
}

#[derive(Clone)]
pub enum Pairings<I: Individual> {
    OneRandomPairing(OneRandomPairing<I>),
    TwoRandomPairing(TwoRandomPairing<I>),
    AsexualPairing(AsexualPairing<I>),
    ThirdFourthNeighborPairing(ThirdFourthNeighborPairing<I>),
    SimilarFitnessPairing(SimilarFitnessPairing<I>),
}

#[derive(Clone)]
pub enum ConstructedPairing {
    TspPairing(Pairings<Sequence>),
    RofaPairing(Pairings<RoutingAndCapacityPlan>),
}

// TODO: this feels wrong, but seems to be the best choice in rust
impl<I: Individual> Pairing<I> for Pairings<I> {
    fn name(&self) -> String {
        match self {
            Self::OneRandomPairing(p) => p.name(),
            Self::TwoRandomPairing(p) => p.name(),
            Self::AsexualPairing(p) => p.name(),
            Self::ThirdFourthNeighborPairing(p) => p.name(),
            Self::SimilarFitnessPairing(p) => p.name(),
        }
    }

    fn pairing_settings(&self) -> PairingSettings {
        match self {
            Self::OneRandomPairing(p) => p.pairing_settings(),
            Self::TwoRandomPairing(p) => p.pairing_settings(),
            Self::AsexualPairing(p) => p.pairing_settings(),
            Self::ThirdFourthNeighborPairing(p) => p.pairing_settings(),
            Self::SimilarFitnessPairing(p) => p.pairing_settings(),
        }
    }

    fn pair(
        &mut self,
        individuals_with_fitness: Vec<(f64, I)>,
        settings: &GeneticAlgorithmSettings,
    ) -> Vec<(I, I)> {
        match self {
            Self::OneRandomPairing(p) => p.pair(individuals_with_fitness, settings),
            Self::TwoRandomPairing(p) => p.pair(individuals_with_fitness, settings),
            Self::AsexualPairing(p) => p.pair(individuals_with_fitness, settings),
            Self::ThirdFourthNeighborPairing(p) => p.pair(individuals_with_fitness, settings),
            Self::SimilarFitnessPairing(p) => p.pair(individuals_with_fitness, settings),
        }
    }
}

#[derive(Clone)]
pub struct OneRandomPairing<I: Individual> {
    pairing_settings: PairingSettings,
    marker: PhantomData<I>,
}

impl<I: Individual> OneRandomPairing<I> {
    pub fn new(pairing_settings: PairingSettings) -> Self {
        assert!(matches!(
            pairing_settings,
            PairingSettings::OneRandomPairing
        ));
        Self {
            pairing_settings,
            marker: PhantomData,
        }
    }
}

impl<I: Individual> Pairing<I> for OneRandomPairing<I> {
    fn name(&self) -> String {
        "orp".into()
    }

    fn pairing_settings(&self) -> PairingSettings {
        self.pairing_settings.clone()
    }

    fn pair(
        &mut self,
        individuals_with_fitness: Vec<(f64, I)>,
        settings: &GeneticAlgorithmSettings,
    ) -> Vec<(I, I)> {
        assert!(
            !individuals_with_fitness.is_empty(),
            "There should be at least one individual"
        );
        let size = settings.population_size();
        let mut pairs = Vec::new();
        for i in 0..size {
            let first_index = i % (individuals_with_fitness.len() - 1);
            let first = individuals_with_fitness
                .get(first_index)
                .expect("Index to choose first is out of bounds")
                .1
                .clone();
            let second = individuals_with_fitness
                .choose(&mut rand::rng())
                .expect("No individual to recombine")
                .1
                .clone();
            pairs.push((first, second));
        }
        pairs
    }
}

#[derive(Clone)]
pub struct TwoRandomPairing<I: Individual> {
    pairing_settings: PairingSettings,
    marker: PhantomData<I>,
}

impl<I: Individual> TwoRandomPairing<I> {
    pub fn new(pairing_settings: PairingSettings) -> Self {
        assert!(matches!(
            pairing_settings,
            PairingSettings::TwoRandomPairing
        ));
        Self {
            pairing_settings,
            marker: PhantomData,
        }
    }
}

impl<I: Individual> Pairing<I> for TwoRandomPairing<I> {
    fn name(&self) -> String {
        "trp".into()
    }

    fn pairing_settings(&self) -> PairingSettings {
        self.pairing_settings.clone()
    }

    fn pair(
        &mut self,
        individuals_with_fitness: Vec<(f64, I)>,
        settings: &GeneticAlgorithmSettings,
    ) -> Vec<(I, I)> {
        assert!(
            !individuals_with_fitness.is_empty(),
            "There should be at least one individual"
        );
        let size = settings.population_size();
        let mut pairs = Vec::new();
        for _ in 0..size {
            let first = individuals_with_fitness
                .choose(&mut rand::rng())
                .expect("No individual to recombine")
                .1
                .clone();
            let second = individuals_with_fitness
                .choose(&mut rand::rng())
                .expect("No individual to recombine")
                .1
                .clone();
            pairs.push((first, second));
        }
        pairs
    }
}

#[derive(Clone)]
pub struct AsexualPairing<I: Individual> {
    pairing_settings: PairingSettings,
    marker: PhantomData<I>,
}

impl<I: Individual> AsexualPairing<I> {
    pub fn new(pairing_settings: PairingSettings) -> Self {
        assert!(matches!(pairing_settings, PairingSettings::AsexualPairing));
        Self {
            pairing_settings,
            marker: PhantomData,
        }
    }
}

impl<I: Individual> Pairing<I> for AsexualPairing<I> {
    fn name(&self) -> String {
        "ap".into()
    }

    fn pairing_settings(&self) -> PairingSettings {
        self.pairing_settings.clone()
    }

    fn pair(
        &mut self,
        individuals_with_fitness: Vec<(f64, I)>,
        settings: &GeneticAlgorithmSettings,
    ) -> Vec<(I, I)> {
        assert!(
            !individuals_with_fitness.is_empty(),
            "There should be at least one individual"
        );
        let size = settings.population_size();
        let mut pairs = Vec::new();
        for i in 0..size {
            let first_index = i % (individuals_with_fitness.len() - 1);
            let first = individuals_with_fitness
                .get(first_index)
                .expect("Index to choose first is out of bounds")
                .1
                .clone();
            let second = first.clone();
            pairs.push((first, second));
        }
        pairs
    }
}

#[derive(Clone)]
pub struct SimilarFitnessPairing<I: Individual> {
    pairing_settings: PairingSettings,
    marker: PhantomData<I>,
}

impl<I: Individual> SimilarFitnessPairing<I> {
    pub fn new(pairing_settings: PairingSettings) -> Self {
        assert!(matches!(
            pairing_settings,
            PairingSettings::SimilarFitnessPairing
        ));
        Self {
            pairing_settings,
            marker: PhantomData,
        }
    }
}

impl<I: Individual> Pairing<I> for SimilarFitnessPairing<I> {
    fn name(&self) -> String {
        "sfp".to_string()
    }

    fn pairing_settings(&self) -> PairingSettings {
        self.pairing_settings.clone()
    }

    fn pair(
        &mut self,
        mut individuals_with_fitness: Vec<(f64, I)>,
        settings: &GeneticAlgorithmSettings,
    ) -> Vec<(I, I)> {
        assert!(
            !individuals_with_fitness.is_empty(),
            "There should be at least one individual"
        );
        let size = settings.population_size();
        let mut pairs = Vec::new();
        individuals_with_fitness.sort_by(|a, b| b.0.total_cmp(&a.0));
        for i in 0..size {
            let first_index = i % (individuals_with_fitness.len() - 1);
            let first = individuals_with_fitness
                .get(first_index)
                .expect("Index to choose first is out of bounds")
                .1
                .clone();
            let second_index = i % (individuals_with_fitness.len() - 1);
            let second = individuals_with_fitness
                .get(second_index)
                .expect("Index to choose first is out of bounds")
                .1
                .clone();
            pairs.push((first, second));
        }
        pairs
    }
}

#[derive(Clone)]
pub struct ThirdFourthNeighborPairing<I: Individual> {
    pairing_settings: PairingSettings,
    marker: PhantomData<I>,
}

impl<I: Individual> ThirdFourthNeighborPairing<I> {
    pub fn new(pairing_settings: PairingSettings) -> Self {
        assert!(matches!(pairing_settings, PairingSettings::NeighborPairing));
        Self {
            pairing_settings,
            marker: PhantomData,
        }
    }
}

impl<I: Individual> Pairing<I> for ThirdFourthNeighborPairing<I> {
    fn name(&self) -> String {
        "tfnp".into()
    }

    fn pairing_settings(&self) -> PairingSettings {
        self.pairing_settings.clone()
    }

    fn pair(
        &mut self,
        individuals_with_fitness: Vec<(f64, I)>,
        settings: &GeneticAlgorithmSettings,
    ) -> Vec<(I, I)> {
        assert!(
            !individuals_with_fitness.is_empty(),
            "There should be at least one individual"
        );
        let size = settings.population_size();
        let mut pairs = Vec::new();
        for i in 0..size {
            let first_index = i % (individuals_with_fitness.len() - 1);
            let first = individuals_with_fitness
                .get(first_index)
                .expect("Index to choose first is out of bounds")
                .1
                .clone();
            let index_distance = rand::random_range(3..=4);
            let second_index =
                (first_index + index_distance) % (individuals_with_fitness.len() - 1);
            let second = individuals_with_fitness
                .get(second_index)
                .expect("No individual to recombine")
                .1
                .clone();
            pairs.push((first, second));
        }
        pairs
    }
}

use std::{
    any::Any,
    collections::HashMap,
    marker::PhantomData,
    sync::atomic::{AtomicU64, Ordering},
};

use rand::{seq::IndexedRandom, Rng};
use shared::{
    settings::{GeneticAlgorithmSettings, PairingSettings, ProblemSettings},
    statistics::Statistic,
};

use crate::problems::{
    rofa::{Rofa, RoutingAndCapacityPlan},
    tsp::{Cities, Sequence},
};

pub type Id = usize;
static ID: AtomicU64 = AtomicU64::new(0);

pub fn get_id() -> Id {
    ID.fetch_add(1, Ordering::Relaxed) as usize
}

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
        let mut individuals = (0..self.genetic_algorithm_settings.population_size())
            .map(|_| self.problem.random_individual())
            .collect();
        for generation in 0..self.genetic_algorithm_settings.generations() {
            individuals = self.step(individuals);
        }
    }

    pub fn get_statistic(self) -> Statistic {
        self.statistic
    }

    fn step(&mut self, individuals: Vec<I>) -> Vec<I> {
        let selected_individuals_with_fitness = self.select(&individuals);
        let recombined_individuals = self.recombine(selected_individuals_with_fitness);
        let mutated_individuals = self.mutate(recombined_individuals);
        mutated_individuals
    }

    fn select<'a>(&mut self, individuals: &'a Vec<I>) -> Vec<(f64, &'a I)> {
        let mut individuals_and_fitness: Vec<(usize, f64, &I)> = individuals
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

    fn recombine(&mut self, individuals_with_fitness: Vec<(f64, &I)>) -> Vec<I> {
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
    fn id(&self) -> Id;
    fn parent_ids(&self) -> (Id, Id);
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
    fn pairing_settings(&self) -> PairingSettings;
    fn pair<'a>(
        &mut self,
        individuals_with_fitness: Vec<(f64, &'a I)>,
        settings: &GeneticAlgorithmSettings,
    ) -> Vec<(&'a I, &'a I)>;
}

#[derive(Clone)]
pub enum Pairings<I: Individual> {
    // random trying to avoid some duplication by choosing the first parent in order
    OneRandomPairing(OneRandomPairing<I>),
    // completely random
    TwoRandomPairing(TwoRandomPairing<I>),
    // only one individual is choosen and recombined with itself
    AsexualPairing(AsexualPairing<I>),
    // pairs are roughly created out of neighbors
    ThirdFourthNeighborPairing(ThirdFourthNeighborPairing<I>),
    // individuals are paired with individuals that are closest in fitness
    SimilarFitnessPairing(SimilarFitnessPairing<I>),
    // individuals with higher fitness are paired more often
    FitnessProportionatePairing(FitnessProportionatePairing<I>),
    // all individuals are recombined once and the upper ones fill the remaining populaton
    ElitePairing(ElitePairing<I>),
    // all individuals are recombined once and the lower ones fill the remaining populaton
    AntiElitePairing(AntiElitePairing<I>),
    // pairing based on spatial distance
    SpatialDistancePairing(SpatialDistancePairing<I, 2>),
}

#[derive(Clone)]
pub enum ConstructedPairing {
    TspPairing(Pairings<Sequence>),
    RofaPairing(Pairings<RoutingAndCapacityPlan>),
}

// TODO: this feels wrong, but seems to be the best choice in rust
impl<I: Individual> Pairing<I> for Pairings<I> {
    fn pairing_settings(&self) -> PairingSettings {
        match self {
            Self::OneRandomPairing(p) => p.pairing_settings(),
            Self::TwoRandomPairing(p) => p.pairing_settings(),
            Self::AsexualPairing(p) => p.pairing_settings(),
            Self::ThirdFourthNeighborPairing(p) => p.pairing_settings(),
            Self::SimilarFitnessPairing(p) => p.pairing_settings(),
            Self::FitnessProportionatePairing(p) => p.pairing_settings(),
            Self::ElitePairing(p) => p.pairing_settings(),
            Self::AntiElitePairing(p) => p.pairing_settings(),
            Self::SpatialDistancePairing(p) => p.pairing_settings(),
        }
    }

    fn pair<'a>(
        &mut self,
        individuals_with_fitness: Vec<(f64, &'a I)>,
        settings: &GeneticAlgorithmSettings,
    ) -> Vec<(&'a I, &'a I)> {
        match self {
            Self::OneRandomPairing(p) => p.pair(individuals_with_fitness, settings),
            Self::TwoRandomPairing(p) => p.pair(individuals_with_fitness, settings),
            Self::AsexualPairing(p) => p.pair(individuals_with_fitness, settings),
            Self::ThirdFourthNeighborPairing(p) => p.pair(individuals_with_fitness, settings),
            Self::SimilarFitnessPairing(p) => p.pair(individuals_with_fitness, settings),
            Self::FitnessProportionatePairing(p) => p.pair(individuals_with_fitness, settings),
            Self::ElitePairing(p) => p.pair(individuals_with_fitness, settings),
            Self::AntiElitePairing(p) => p.pair(individuals_with_fitness, settings),
            Self::SpatialDistancePairing(p) => p.pair(individuals_with_fitness, settings),
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
    fn pairing_settings(&self) -> PairingSettings {
        self.pairing_settings.clone()
    }

    fn pair<'a>(
        &mut self,
        individuals_with_fitness: Vec<(f64, &'a I)>,
        settings: &GeneticAlgorithmSettings,
    ) -> Vec<(&'a I, &'a I)> {
        assert!(
            !individuals_with_fitness.is_empty(),
            "There should be at least one individual"
        );
        let size = settings.population_size();
        let mut pairs = Vec::new();
        for i in 0..size {
            let first_index = i % individuals_with_fitness.len();
            let first = individuals_with_fitness
                .get(first_index)
                .expect("Index to choose first is out of bounds")
                .1;
            let second = individuals_with_fitness
                .choose(&mut rand::rng())
                .expect("No individual to recombine")
                .1;
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
    fn pairing_settings(&self) -> PairingSettings {
        self.pairing_settings.clone()
    }

    fn pair<'a>(
        &mut self,
        individuals_with_fitness: Vec<(f64, &'a I)>,
        settings: &GeneticAlgorithmSettings,
    ) -> Vec<(&'a I, &'a I)> {
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
                .1;
            let second = individuals_with_fitness
                .choose(&mut rand::rng())
                .expect("No individual to recombine")
                .1;
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
    fn pairing_settings(&self) -> PairingSettings {
        self.pairing_settings.clone()
    }

    fn pair<'a>(
        &mut self,
        individuals_with_fitness: Vec<(f64, &'a I)>,
        settings: &GeneticAlgorithmSettings,
    ) -> Vec<(&'a I, &'a I)> {
        assert!(
            !individuals_with_fitness.is_empty(),
            "There should be at least one individual"
        );
        let size = settings.population_size();
        let mut pairs = Vec::new();
        for i in 0..size {
            let first_index = i % individuals_with_fitness.len();
            let first = individuals_with_fitness
                .get(first_index)
                .expect("Index to choose first is out of bounds")
                .1;
            let second = first;
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
    fn pairing_settings(&self) -> PairingSettings {
        self.pairing_settings.clone()
    }

    fn pair<'a>(
        &mut self,
        mut individuals_with_fitness: Vec<(f64, &'a I)>,
        settings: &GeneticAlgorithmSettings,
    ) -> Vec<(&'a I, &'a I)> {
        assert!(
            !individuals_with_fitness.is_empty(),
            "There should be at least one individual"
        );
        let size = settings.population_size();
        let mut pairs = Vec::new();
        individuals_with_fitness.sort_by(|a, b| b.0.total_cmp(&a.0));
        for i in 0..size {
            let first_index = i % individuals_with_fitness.len();
            let first = individuals_with_fitness
                .get(first_index)
                .expect("Index to choose first is out of bounds")
                .1;
            let second_index = (i + 1) % individuals_with_fitness.len();
            let second = individuals_with_fitness
                .get(second_index)
                .expect("Index to choose first is out of bounds")
                .1;
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
    fn pairing_settings(&self) -> PairingSettings {
        self.pairing_settings.clone()
    }

    fn pair<'a>(
        &mut self,
        individuals_with_fitness: Vec<(f64, &'a I)>,
        settings: &GeneticAlgorithmSettings,
    ) -> Vec<(&'a I, &'a I)> {
        assert!(
            !individuals_with_fitness.is_empty(),
            "There should be at least one individual"
        );
        let size = settings.population_size();
        let mut pairs = Vec::new();
        for i in 0..size {
            let first_index = i % individuals_with_fitness.len();
            let first = individuals_with_fitness
                .get(first_index)
                .expect("Index to choose first is out of bounds")
                .1;
            let index_distance = rand::random_range(3..=4);
            let second_index = (first_index + index_distance) % individuals_with_fitness.len();
            let second = individuals_with_fitness
                .get(second_index)
                .expect("No individual to recombine")
                .1;
            pairs.push((first, second));
        }
        pairs
    }
}

#[derive(Clone)]
pub struct FitnessProportionatePairing<I: Individual> {
    pairing_settings: PairingSettings,
    marker: PhantomData<I>,
}

impl<I: Individual> FitnessProportionatePairing<I> {
    pub fn new(pairing_settings: PairingSettings) -> Self {
        assert!(matches!(
            pairing_settings,
            PairingSettings::FitnessProportionatePairing
        ));
        Self {
            pairing_settings,
            marker: PhantomData,
        }
    }

    fn next_to_pair<'a, 'b>(
        individuals_with_fitness: &'b mut Vec<(f64, &'a I)>,
    ) -> &'b mut (f64, &'a I) {
        let next_to_pair: &mut (f64, &I) = individuals_with_fitness
            .iter_mut()
            .max_by(|a, b| a.0.total_cmp(&b.0))
            .expect("Could not get relative fitness max of individuals");
        next_to_pair
    }
}

impl<I: Individual> Pairing<I> for FitnessProportionatePairing<I> {
    fn pairing_settings(&self) -> PairingSettings {
        self.pairing_settings.clone()
    }

    fn pair<'a>(
        &mut self,
        mut individuals_with_fitness: Vec<(f64, &'a I)>,
        settings: &GeneticAlgorithmSettings,
    ) -> Vec<(&'a I, &'a I)> {
        assert!(
            !individuals_with_fitness.is_empty(),
            "There should be at least one individual"
        );
        let size = settings.population_size();
        let mut pairs = Vec::new();
        let total_fitness_absolute = individuals_with_fitness
            .iter()
            .map(|i| i.0)
            .reduce(|acc, e| acc + e)
            .expect("Could not sum individuals fitness")
            .abs();
        individuals_with_fitness
            .iter_mut()
            .for_each(|c| *c = (c.0 * 2.0 * size as f64 / total_fitness_absolute, c.1));
        for _ in 0..size {
            let first_with_fitness = Self::next_to_pair(&mut individuals_with_fitness);
            first_with_fitness.0 -= 1.0;
            let first = first_with_fitness.1;
            let second_with_fitness = Self::next_to_pair(&mut individuals_with_fitness);
            second_with_fitness.0 -= 1.0;
            let second = second_with_fitness.1;
            pairs.push((first, second));
        }
        pairs
    }
}

#[derive(Clone)]
pub struct ElitePairing<I: Individual> {
    pairing_settings: PairingSettings,
    marker: PhantomData<I>,
    elite_percentage: usize,
}

impl<I: Individual> ElitePairing<I> {
    pub fn new(pairing_settings: PairingSettings) -> Self {
        assert!(matches!(pairing_settings, PairingSettings::ElitePairing));
        Self {
            pairing_settings,
            marker: PhantomData,
            elite_percentage: 30,
        }
    }
}

impl<I: Individual> Pairing<I> for ElitePairing<I> {
    fn pairing_settings(&self) -> PairingSettings {
        self.pairing_settings.clone()
    }

    fn pair<'a>(
        &mut self,
        mut individuals_with_fitness: Vec<(f64, &'a I)>,
        settings: &GeneticAlgorithmSettings,
    ) -> Vec<(&'a I, &'a I)> {
        assert!(
            !individuals_with_fitness.is_empty(),
            "There should be at least one individual"
        );
        let size = settings.population_size();
        let mut pairs = Vec::new();
        for i in (0..individuals_with_fitness.len()).step_by(2) {
            let first_index = i;
            let second_index = i + 1;
            let first = individuals_with_fitness
                .get(first_index)
                .expect("Could not get first individual")
                .1;
            let second = individuals_with_fitness
                .get(second_index)
                .expect("Could not get second individual")
                .1;
            pairs.push((first, second));
        }
        individuals_with_fitness.sort_by(|a, b| b.0.total_cmp(&a.0));
        let number_elite =
            (0.01 * self.elite_percentage as f64 * individuals_with_fitness.len() as f64).ceil()
                as usize;
        for i in 0..(size - pairs.len()) {
            let first_index = i % number_elite;
            let second_index = (i + 1) % number_elite;
            let first = individuals_with_fitness
                .get(first_index)
                .expect("Could not get first individual")
                .1;
            let second = individuals_with_fitness
                .get(second_index)
                .expect("Could not get second individual")
                .1;
            pairs.push((first, second));
        }
        pairs
    }
}

#[derive(Clone)]
pub struct AntiElitePairing<I: Individual> {
    pairing_settings: PairingSettings,
    marker: PhantomData<I>,
    anti_elite_percentage: usize,
}

impl<I: Individual> AntiElitePairing<I> {
    pub fn new(pairing_settings: PairingSettings) -> Self {
        assert!(matches!(
            pairing_settings,
            PairingSettings::AntiElitePairing
        ));
        Self {
            pairing_settings,
            marker: PhantomData,
            anti_elite_percentage: 30,
        }
    }
}

impl<I: Individual> Pairing<I> for AntiElitePairing<I> {
    fn pairing_settings(&self) -> PairingSettings {
        self.pairing_settings.clone()
    }

    fn pair<'a>(
        &mut self,
        mut individuals_with_fitness: Vec<(f64, &'a I)>,
        settings: &GeneticAlgorithmSettings,
    ) -> Vec<(&'a I, &'a I)> {
        assert!(
            !individuals_with_fitness.is_empty(),
            "There should be at least one individual"
        );
        let size = settings.population_size();
        let mut pairs = Vec::new();
        for i in (0..individuals_with_fitness.len()).step_by(2) {
            let first_index = i;
            let second_index = i + 1;
            let first = individuals_with_fitness
                .get(first_index)
                .expect("Could not get first individual")
                .1;
            let second = individuals_with_fitness
                .get(second_index)
                .expect("Could not get second individual")
                .1;
            pairs.push((first, second));
        }
        individuals_with_fitness.sort_by(|a, b| a.0.total_cmp(&b.0));
        let number_anti_elite =
            (0.01 * self.anti_elite_percentage as f64 * individuals_with_fitness.len() as f64)
                .ceil() as usize;
        for i in 0..(size - pairs.len()) {
            let first_index = i % number_anti_elite;
            let second_index = (i + 1) % number_anti_elite;
            let first = individuals_with_fitness
                .get(first_index)
                .expect("Could not get first individual")
                .1;
            let second = individuals_with_fitness
                .get(second_index)
                .expect("Could not get first individual")
                .1;
            pairs.push((first, second));
        }
        pairs
    }
}

#[derive(Clone)]
pub struct SpatialDistancePairing<I: Individual, const DIMENSIONS: usize> {
    pairing_settings: PairingSettings,
    marker: PhantomData<I>,
    desired_individual_distance_percentage: usize,
    space: Option<Space<DIMENSIONS>>,
}

impl<I: Individual, const DIMENSIONS: usize> SpatialDistancePairing<I, DIMENSIONS> {
    pub fn new(pairing_settings: PairingSettings) -> Self {
        let desired_individual_distance_percentage = match pairing_settings {
            PairingSettings::SpatialDistancePairing {
                desired_individual_distance_percentage,
            } => desired_individual_distance_percentage,
            _ => panic!("Invalid pairing settings"),
        };
        Self {
            pairing_settings,
            marker: PhantomData,
            desired_individual_distance_percentage,
            space: None,
        }
    }
}

impl<I: Individual, const DIMENSIONS: usize> Pairing<I> for SpatialDistancePairing<I, DIMENSIONS> {
    fn pairing_settings(&self) -> PairingSettings {
        self.pairing_settings.clone()
    }

    fn pair<'a>(
        &mut self,
        mut individuals_with_fitness: Vec<(f64, &'a I)>,
        settings: &GeneticAlgorithmSettings,
    ) -> Vec<(&'a I, &'a I)> {
        assert!(
            !individuals_with_fitness.is_empty(),
            "There should be at least one individual"
        );
        self.space = match &mut self.space {
            None => {
                let ids = individuals_with_fitness.iter().map(|i| i.1.id()).collect();
                Some(Space::new_at_origin(ids))
            }
            Some(s) => {
                let id_relationships = individuals_with_fitness
                    .iter()
                    .map(|i| (i.1.id(), i.1.parent_ids()))
                    .collect();
                Some(s.updated_from_id_relationships(id_relationships))
            }
        };
        let size = settings.population_size();
        let mut pairs = Vec::new();
        for i in 0..size {
            let first_index = i % individuals_with_fitness.len();
            let first_individual = individuals_with_fitness
                .get(first_index)
                .expect("Could not get first individual from index")
                .1;
            let desired_individual_distance_number = 1
                + (0.01
                    * self.desired_individual_distance_percentage as f64
                    * (individuals_with_fitness.len() as f64 - 2.0)) as usize;
            let second_id = &self
                .space
                .as_ref()
                .expect("There should be a space available")
                .get_id_of_close_individual(
                    first_individual.id(),
                    desired_individual_distance_number,
                );
            let second_individual = individuals_with_fitness
                .iter()
                .find(|(_, i)| i.id() == *second_id)
                .expect("Could not find individual by id")
                .1;
            pairs.push((first_individual, second_individual));
        }
        pairs
    }
}

#[derive(Clone, Debug)]
struct Position<const N: usize> {
    coordinates: Vec<f64>,
}

impl<const N: usize> Position<N> {
    fn origin() -> Self {
        let coordinates = vec![0.0; N];
        Self { coordinates }
    }

    fn middle(first: &Self, second: &Self) -> Self {
        let mut new_coordinates = Vec::new();
        for (first_coordinate, second_coordinate) in
            first.coordinates.iter().zip(&second.coordinates)
        {
            let new_coordinate = (first_coordinate + second_coordinate) / 2.0;
            new_coordinates.push(new_coordinate);
        }
        Self {
            coordinates: new_coordinates,
        }
    }

    fn distance(first: &Self, second: &Self) -> f64 {
        let squared_distance: f64 = first
            .coordinates
            .iter()
            .zip(&second.coordinates)
            .map(|(f_c, s_c)| (f_c - s_c).powi(2))
            .sum();
        squared_distance.sqrt()
    }
}

#[derive(Clone, Debug)]
struct Space<const N: usize> {
    positions: HashMap<Id, Position<N>>,
}

impl<const N: usize> Space<N> {
    fn new_at_origin(ids: Vec<Id>) -> Self {
        let mut positions = HashMap::new();
        for id in ids {
            positions.insert(id, Position::origin());
        }
        Self { positions }
    }

    pub fn updated_from_id_relationships(&mut self, id_relationships: Vec<(Id, (Id, Id))>) -> Self {
        let mut new_positions = HashMap::new();
        for (child_id, (first_parent_id, second_parent_id)) in id_relationships {
            let first_position = self
                .positions
                .get(&first_parent_id)
                .expect("First position not found in space");
            let second_position = self
                .positions
                .get(&second_parent_id)
                .expect("Second position not found in space");
            let new_position = Position::middle(first_position, second_position);
            new_positions.insert(child_id, new_position);
        }
        Self {
            positions: new_positions,
        }
    }

    fn get_id_of_close_individual(
        &self,
        individual_id: Id,
        desired_individual_distance_number: usize,
    ) -> Id {
        let position_individual = self
            .positions
            .get(&individual_id)
            .expect("Individual not in space positions");
        let mut distances_and_ids = Vec::new();
        for (&id_other, position_other) in &self.positions {
            let distance = Position::distance(position_individual, position_other);
            distances_and_ids.push((distance, id_other));
        }
        distances_and_ids.sort_by(|a, b| {
            a.0.partial_cmp(&b.0)
                .expect("Could not order ids by distance")
        });
        assert!(!distances_and_ids.is_empty(), "Distances can not be empty");
        distances_and_ids
            .get(desired_individual_distance_number)
            .expect("Could not get individual with desired spatial distance")
            .1
    }
}

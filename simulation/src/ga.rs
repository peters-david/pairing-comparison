use std::{
    any::Any,
    collections::BTreeMap,
    marker::PhantomData,
    sync::atomic::{AtomicU64, Ordering},
};

use indicatif::ProgressBar;
use itertools::Itertools;
use rand::{prelude::SliceRandom, rngs::StdRng, RngExt};
use rand_distr::{Distribution, Normal};
use shared::{
    settings::{GeneticAlgorithmSettings, IndividualQuantity, PairingSettings, ProblemSettings},
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

    pub fn run(&mut self, rng: &mut StdRng, unique_number: usize, progress_bar: ProgressBar) {
        let base_individual = self.problem.random_individual(rng);
        let mut individuals = (0..self.genetic_algorithm_settings.population_size())
            .map(|_| {
                let mut individual = base_individual.clone();
                if rng.random_bool(0.1) {
                    individual.mutate(rng, &self.problem);
                }
                individual
            })
            .collect();
        for generation in 0..self.genetic_algorithm_settings.generations() {
            progress_bar.set_message(format!("uid{}/gen{}", unique_number, generation));
            individuals = self.step(rng, individuals);
        }
    }

    pub fn get_statistic(self) -> Statistic {
        self.statistic
    }

    fn step(&mut self, rng: &mut StdRng, individuals: Vec<I>) -> Vec<I> {
        let selected_individuals_with_fitness = self.select(&individuals);
        let recombined_individuals = self.recombine(rng, selected_individuals_with_fitness);
        let mutated_individuals = self.mutate(rng, recombined_individuals);
        mutated_individuals
    }

    fn select<'a>(&mut self, individuals: &'a Vec<I>) -> Vec<(f64, &'a I)> {
        let mut individuals_and_fitness: Vec<(usize, f64, &I)> = individuals
            .into_iter()
            .enumerate()
            .map(|(i, individual)| (i, self.problem.fitness(&individual), individual))
            .collect();
        let fitness_values = individuals_and_fitness.iter().map(|(_, f, _)| *f).collect();
        self.statistic.append_fitness_values(fitness_values);
        individuals_and_fitness.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .expect("Tried to compare invalid numbers")
        });
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

    fn recombine(&mut self, rng: &mut StdRng, individuals_with_fitness: Vec<(f64, &I)>) -> Vec<I> {
        let pairs = self.pairing.pair(
            rng,
            individuals_with_fitness,
            &self.genetic_algorithm_settings,
        );
        let recombined_individuals = pairs
            .iter()
            .map(|(a, b)| I::crossover(rng, a, b, &self.problem))
            .collect();
        recombined_individuals
    }

    fn mutate(&self, rng: &mut StdRng, mut individuals: Vec<I>) -> Vec<I> {
        if individuals.len() == 2 {
            // 1+1 evolutionary
            for _ in 0..self.genetic_algorithm_settings.mutation_strength() {
                individuals[1].mutate(rng, &self.problem);
            }
        } else {
            for individual in &mut individuals {
                if self.genetic_algorithm_settings.mutation_rate() >= rng.random_range(0.0..=1.0) {
                    for _ in 0..self.genetic_algorithm_settings.mutation_strength() {
                        individual.mutate(rng, &self.problem);
                    }
                }
            }
        }
        individuals
    }
}

pub trait Individual: Any + Clone {
    type Problem: Problem<Individual = Self>;
    fn crossover(rng: &mut StdRng, first: &Self, second: &Self, problem: &Self::Problem) -> Self;
    fn mutate(&mut self, rng: &mut StdRng, problem: &Self::Problem);
    fn id(&self) -> Id;
    fn parent_ids(&self) -> (Id, Id);
}

pub trait Problem {
    type Individual: Individual<Problem = Self>;
    fn random(rng: &mut StdRng, problem_settings: &ProblemSettings) -> Self;
    fn random_individual(&self, rng: &mut StdRng) -> Self::Individual;
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
        rng: &mut StdRng,
        individuals_with_fitness: Vec<(f64, &'a I)>,
        settings: &GeneticAlgorithmSettings,
    ) -> Vec<(&'a I, &'a I)>;
}

#[derive(Clone)]
pub enum Pairings<I: Individual> {
    // random
    RandomPairing(RandomPairing<I>),
    // only one individual is choosen and recombined with itself
    AsexualPairing(AsexualPairing<I>),
    // individuals are paired with individuals that are closest in fitness
    SimilarFitnessPairing(SimilarFitnessPairing<I>),
    // pairing based on spatial distance
    SpatialDistancePairing(SpatialDistancePairing<I, 1>),
}

#[derive(Clone)]
pub enum ConstructedPairing {
    TspPairing(Pairings<Sequence>),
    RofaPairing(Pairings<RoutingAndCapacityPlan>),
}

impl<I: Individual> Pairing<I> for Pairings<I> {
    fn pairing_settings(&self) -> PairingSettings {
        match self {
            Self::RandomPairing(p) => p.pairing_settings(),
            Self::AsexualPairing(p) => p.pairing_settings(),
            Self::SimilarFitnessPairing(p) => p.pairing_settings(),
            Self::SpatialDistancePairing(p) => p.pairing_settings(),
        }
    }

    fn pair<'a>(
        &mut self,
        rng: &mut StdRng,
        individuals_with_fitness: Vec<(f64, &'a I)>,
        settings: &GeneticAlgorithmSettings,
    ) -> Vec<(&'a I, &'a I)> {
        match self {
            Self::RandomPairing(p) => p.pair(rng, individuals_with_fitness, settings),
            Self::AsexualPairing(p) => p.pair(rng, individuals_with_fitness, settings),
            Self::SimilarFitnessPairing(p) => p.pair(rng, individuals_with_fitness, settings),
            Self::SpatialDistancePairing(p) => p.pair(rng, individuals_with_fitness, settings),
        }
    }
}

#[derive(Clone)]
pub struct RandomPairing<I: Individual> {
    pairing_settings: PairingSettings,
    marker: PhantomData<I>,
}

impl<I: Individual> RandomPairing<I> {
    pub fn new(pairing_settings: PairingSettings) -> Self {
        assert!(matches!(
            pairing_settings,
            PairingSettings::RandomPairing { .. }
        ));
        Self {
            pairing_settings,
            marker: PhantomData,
        }
    }
}

impl<I: Individual> Pairing<I> for RandomPairing<I> {
    fn pairing_settings(&self) -> PairingSettings {
        self.pairing_settings.clone()
    }

    fn pair<'a>(
        &mut self,
        rng: &mut StdRng,
        individuals_with_fitness: Vec<(f64, &'a I)>,
        settings: &GeneticAlgorithmSettings,
    ) -> Vec<(&'a I, &'a I)> {
        assert!(
            !individuals_with_fitness.is_empty(),
            "There should be at least one individual"
        );
        let size = settings.population_size();
        let mut parents = get_parents(
            rng,
            self.pairing_settings.quantity(),
            individuals_with_fitness,
            size,
        );
        let mut pairs = Vec::new();
        parents.shuffle(rng);
        for _ in 0..size {
            let first = parents.remove(0).1;
            let second = parents.remove(0).1;
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
        assert!(matches!(
            pairing_settings,
            PairingSettings::AsexualPairing { .. }
        ));
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
        rng: &mut StdRng,
        individuals_with_fitness: Vec<(f64, &'a I)>,
        settings: &GeneticAlgorithmSettings,
    ) -> Vec<(&'a I, &'a I)> {
        assert!(
            !individuals_with_fitness.is_empty(),
            "There should be at least one individual"
        );
        let size = settings.population_size();
        let mut parents = get_parents(
            rng,
            self.pairing_settings.quantity(),
            individuals_with_fitness,
            size,
        );
        let mut pairs = Vec::new();
        parents.sort_by(|a, b| a.1.id().partial_cmp(&b.1.id()).expect("Cannot sort by id"));
        for i in 0..size {
            let first = parents.remove(0).1;
            let second = parents.remove(0).1;
            assert!(
                first.id() == second.id(),
                "Parents must match in asexual pairing"
            );
            pairs.push((first, second));
        }
        pairs
    }
}

#[derive(Clone)]
pub struct SimilarFitnessPairing<I: Individual> {
    pairing_settings: PairingSettings,
    marker: PhantomData<I>,
    similarity: usize,
}

impl<I: Individual> SimilarFitnessPairing<I> {
    pub fn new(pairing_settings: PairingSettings) -> Self {
        let similarity = match pairing_settings {
            PairingSettings::SimilarFitnessPairing {
                quantity,
                similarity,
            } => similarity,
            _ => panic!("Invalid pairing settings"),
        };
        Self {
            pairing_settings,
            marker: PhantomData,
            similarity,
        }
    }
}

impl<I: Individual> Pairing<I> for SimilarFitnessPairing<I> {
    fn pairing_settings(&self) -> PairingSettings {
        self.pairing_settings.clone()
    }

    fn pair<'a>(
        &mut self,
        rng: &mut StdRng,
        mut individuals_with_fitness: Vec<(f64, &'a I)>,
        settings: &GeneticAlgorithmSettings,
    ) -> Vec<(&'a I, &'a I)> {
        assert!(
            !individuals_with_fitness.is_empty(),
            "There should be at least one individual"
        );
        let size = settings.population_size();
        let mut parents = get_parents(
            rng,
            self.pairing_settings.quantity(),
            individuals_with_fitness,
            size,
        );
        let mut pairs = Vec::new();
        parents.sort_by(|a, b| b.0.total_cmp(&a.0));
        for i in 0..size {
            let first_index = match i < parents.len() {
                true => i,
                false => rng.random_range(0..parents.len()),
            };
            let distance =
                ((1.0 - 0.01 * self.similarity as f64) * (parents.len() as f64 - 1.0)) as usize;
            let second_index = (first_index + distance).max(0).min(parents.len() - 2);
            let first = parents.remove(first_index).1;
            let second = parents.remove(second_index).1;
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
                quantity,
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
        rng: &mut StdRng,
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
                Some(s.updated_from_id_relationships(rng, id_relationships))
            }
        };
        let size = settings.population_size();
        let mut parents = get_parents(
            rng,
            self.pairing_settings.quantity(),
            individuals_with_fitness,
            size,
        );
        let desired_individual_distance_number = 1
            + (0.01
                * self.desired_individual_distance_percentage as f64
                * (parents.len() as f64 - 2.0)) as usize;
        let mut pairs = Vec::new();
        for i in 0..size {
            let first_index = rng.random_range(0..parents.len());
            let first = parents.remove(first_index).1;
            let existing_ids = parents.iter().map(|p| p.1.id()).collect();
            let second_id = &self
                .space
                .as_ref()
                .expect("There should be a space available")
                .get_id_of_close_individual(
                    first.id(),
                    desired_individual_distance_number,
                    existing_ids,
                );
            let second = parents
                .iter()
                .find(|(_, i)| i.id() == *second_id)
                .expect("Could not find individual by id")
                .1;
            if let Some(index) = parents.iter().position(|p| p.1.id() == *second_id) {
                parents.remove(index);
            }
            pairs.push((first, second));
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

    fn from_coordinates(coordinates: Vec<f64>) -> Self {
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

    fn normalize(mut self) -> Self {
        let length = self.coordinates.iter().map(|v| v * v).sum::<f64>().sqrt();
        for v in &mut self.coordinates {
            *v /= length;
        }
        self
    }

    fn mutate(self, rng: &mut StdRng) -> Self {
        let normal = Normal::new(0.0, 1.0).expect("Could not create normal");
        let mutation_coordinates: Vec<f64> =
            (0..N).map(|_| normal.sample(rng)).collect::<Vec<f64>>();
        let mutation = Self::from_coordinates(mutation_coordinates).normalize();
        Self::sum(self, mutation)
    }

    fn sum(first: Self, second: Self) -> Self {
        let coordinates = first
            .coordinates
            .iter()
            .zip(second.coordinates)
            .map(|(f, s)| f + s)
            .collect();
        Self { coordinates }
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
    positions: BTreeMap<Id, Position<N>>,
}

impl<const N: usize> Space<N> {
    fn new_at_origin(ids: Vec<Id>) -> Self {
        let mut positions = BTreeMap::new();
        for id in ids {
            positions.insert(id, Position::origin());
        }
        Self { positions }
    }

    pub fn updated_from_id_relationships(
        &mut self,
        rng: &mut StdRng,
        id_relationships: Vec<(Id, (Id, Id))>,
    ) -> Self {
        let mut new_positions = BTreeMap::new();
        for (child_id, (first_parent_id, second_parent_id)) in id_relationships {
            let first_position = self
                .positions
                .get(&first_parent_id)
                .expect("First position not found in space");
            let second_position = self
                .positions
                .get(&second_parent_id)
                .expect("Second position not found in space");
            let new_position = Position::middle(first_position, second_position).mutate(rng);
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
        existing_ids: Vec<Id>,
    ) -> Id {
        let position_individual = self
            .positions
            .get(&individual_id)
            .expect("Individual not in space positions");
        let mut distances_and_ids = Vec::new();
        for (&id_other, position_other) in &self.positions {
            if existing_ids.contains(&id_other) {
                let distance = Position::distance(position_individual, position_other);
                distances_and_ids.push((distance, id_other));
            }
        }
        assert!(
            !distances_and_ids.is_empty(),
            "Distances and ids can not be empty"
        );
        distances_and_ids.sort_by(|a, b| {
            a.0.partial_cmp(&b.0)
                .expect("Could not order ids by distance")
        });
        assert!(!distances_and_ids.is_empty(), "Distances can not be empty");
        let mut delta_distances_and_ids = Vec::new();
        for delta in 0..=desired_individual_distance_number {
            let new_desired_disance = desired_individual_distance_number + delta;
            match distances_and_ids.get(new_desired_disance) {
                None => {}
                Some(&d_a_i) => delta_distances_and_ids.push(d_a_i),
            };
            let new_desired_disance = desired_individual_distance_number - delta;
            match distances_and_ids.get(new_desired_disance) {
                None => {}
                Some(&d_a_i) => delta_distances_and_ids.push(d_a_i),
            };
        }
        delta_distances_and_ids.remove(0).1
    }
}

pub fn get_parents<'a, T>(
    rng: &mut StdRng,
    quantity: &IndividualQuantity,
    mut individuals_with_fitness: Vec<(f64, &'a T)>,
    number_parents: usize,
) -> Vec<(f64, &'a T)> {
    assert!(
        individuals_with_fitness.len() <= number_parents,
        "At least as many parents must be wanted as individuals in list"
    );
    let mut first_parents = Vec::with_capacity(number_parents);
    first_parents.extend(individuals_with_fitness.clone());
    match quantity {
        IndividualQuantity::Random => {
            for _ in 0..(number_parents - individuals_with_fitness.len()) {
                let random_index = rng.random_range(0..individuals_with_fitness.len());
                let random_individual = individuals_with_fitness
                    .get(random_index)
                    .expect("Cannot access individual");
                first_parents.push(*random_individual);
            }
        }
        IndividualQuantity::Elite { percentage } => {
            individuals_with_fitness
                .sort_by(|a, b| b.0.partial_cmp(&a.0).expect("Cannot order fitness"));
            let elite_individuals =
                (*percentage as f64 * individuals_with_fitness.len() as f64).ceil() as usize;
            individuals_with_fitness.truncate(elite_individuals);
            for i in 0..(number_parents - individuals_with_fitness.len()) {
                let elite_index = match individuals_with_fitness.len() {
                    1 => 0,
                    p => i % (p - 1),
                };
                let elite_individual = individuals_with_fitness
                    .get(elite_index)
                    .expect("Cannot access individual");
                first_parents.push(*elite_individual);
            }
        }
        IndividualQuantity::AntiElite { percentage } => {
            individuals_with_fitness
                .sort_by(|a, b| a.0.partial_cmp(&b.0).expect("Cannot order fitness"));
            let anti_elite_individuals =
                (*percentage as f64 * individuals_with_fitness.len() as f64).ceil() as usize;
            individuals_with_fitness.truncate(anti_elite_individuals);
            for i in 0..(number_parents - individuals_with_fitness.len()) {
                let anti_elite_index = i % (individuals_with_fitness.len() - 1);
                let anti_elite_individual = individuals_with_fitness
                    .get(anti_elite_index)
                    .expect("Cannot access individual");
                first_parents.push(*anti_elite_individual);
            }
        }
        IndividualQuantity::FitnessProportionate => {
            let total_fitness_absolute = individuals_with_fitness
                .iter()
                .map(|i| i.0)
                .reduce(|acc, e| acc + e)
                .expect("Could not sum individuals fitness")
                .abs();
            let mut individuals_with_fitness_total: Vec<(f64, f64, &'a T)> =
                individuals_with_fitness
                    .into_iter()
                    .map(|(f, i)| (f, f, i))
                    .collect();
            individuals_with_fitness_total.iter_mut().for_each(|c| {
                *c = (
                    c.0 * number_parents as f64 / total_fitness_absolute,
                    c.1,
                    c.2,
                )
            });
            for _ in 0..(number_parents - individuals_with_fitness_total.len()) {
                let max = individuals_with_fitness_total
                    .iter_mut()
                    .max_by(|a, b| a.0.total_cmp(&b.0))
                    .expect("Could not get relative fitness max of individuals");
                max.0 -= 1.0;
                first_parents.push((max.1, max.2));
            }
        }
    };
    assert!(
        first_parents.len() == number_parents,
        "Must return number parents"
    );
    first_parents.extend(first_parents.clone());
    first_parents
}

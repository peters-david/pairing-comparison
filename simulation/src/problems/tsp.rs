use std::any::Any;

use rand::seq::{IteratorRandom, SliceRandom};

use shared::settings::ProblemSettings;

use crate::{
    ga::{get_id, Id, Individual, Problem},
    point::Point,
};

#[derive(Debug, Clone)]
pub struct Cities {
    positions: Vec<Point>,
    problem_settings: ProblemSettings,
}

impl Cities {
    pub fn random(problem_settings: &ProblemSettings) -> Self {
        let &size = match problem_settings {
            ProblemSettings::Tsp { size } => size,
            _ => panic!("Travelling Salesman Problem requires different kind of settings"),
        };
        let positions = (0..size).map(|_| Point::random_01()).collect();
        Self {
            positions,
            problem_settings: problem_settings.clone(),
        }
    }

    fn get_position_at_number(&self, number: usize) -> &Point {
        self.positions
            .get(number)
            .expect("Tried to access positions out of bounds")
    }

    fn distance(&self, sequence: &Sequence) -> f64 {
        assert!(self.positions.len() == sequence.len());
        let mut total_distance = 0.0;
        for i in 0..sequence.len() {
            let first_index = i;
            let second_index = (i + 1) % (sequence.len() - 1);
            let first = self.get_position_at_number(sequence.get_number_at_index(first_index));
            let second = self.get_position_at_number(sequence.get_number_at_index(second_index));
            let distance = Point::distance(first, second);
            total_distance += distance;
        }
        total_distance
    }
}

impl Problem for Cities {
    type Individual = Sequence;

    fn random(problem_settings: &ProblemSettings) -> Self {
        Self::random(problem_settings)
    }

    fn random_individual(&self) -> Self::Individual {
        Sequence::random(self.positions.len())
    }

    fn fitness(&self, individual: &Self::Individual) -> f64 {
        let sequence = (individual as &dyn Any)
            .downcast_ref::<Sequence>()
            .expect("Cannot downcast individual to sequence");
        -self.distance(sequence)
    }

    fn problem_settings(&self) -> ProblemSettings {
        self.problem_settings.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Sequence {
    numbers: Vec<usize>,
    id: Id,
    parent_ids: Option<(Id, Id)>,
}

impl Sequence {
    pub fn random(size: usize) -> Self {
        let mut numbers = (0..size).collect::<Vec<usize>>();
        numbers.shuffle(&mut rand::rng());
        let id = get_id();
        let parent_ids = None;
        Self {
            numbers,
            id,
            parent_ids,
        }
    }

    fn len(&self) -> usize {
        self.numbers.len()
    }

    fn get_number_at_index(&self, index: usize) -> usize {
        *self
            .numbers
            .get(index)
            .expect("Tried to access numbers out of bounds")
    }

    fn get_index_of_number(&self, number: usize) -> usize {
        self.numbers
            .iter()
            .position(|&n| n == number)
            .expect("Tried to get index of non-existing number")
    }

    fn get_surrounding_numbers(&self, number: usize) -> (usize, usize) {
        let size = self.len();
        let number_index = self.get_index_of_number(number);
        let previous_index = (number_index + size - 1) % (size - 1); // TODO: wrapper?
        let following_index = (number_index + 1) % (size - 1);
        let previous_number = self.get_number_at_index(previous_index);
        let following_number = self.get_number_at_index(following_index);
        (previous_number, following_number)
    }
}

impl Individual for Sequence {
    type Problem = Cities;

    fn crossover(first: &Self, second: &Self, problem: &Self::Problem) -> Self {
        assert!(
            first.len() == second.len(),
            "Sequences need to be the same length"
        );
        let size = first.len();
        let mut new_numbers = Vec::new();
        let first_number = first.get_number_at_index(0);
        new_numbers.push(first_number);
        let second_number = first.get_number_at_index(1);
        new_numbers.push(second_number);
        let mut i = 0;
        while new_numbers.len() < size {
            let newest_number = *new_numbers.last().expect("New numbers are empty");
            let (previous_number_first, following_number_first) =
                first.get_surrounding_numbers(newest_number);
            let (previous_number_second, following_number_second) =
                second.get_surrounding_numbers(newest_number);
            let random_new_number = (0..size)
                .filter(|n| !new_numbers.contains(n))
                .choose(&mut rand::rng())
                .expect("Unexpected no number left");
            let possible_numbers = match i % 2 {
                1 => vec![
                    previous_number_first,
                    following_number_first,
                    previous_number_second,
                    following_number_second,
                    random_new_number,
                ],
                _ => vec![
                    previous_number_second,
                    following_number_second,
                    previous_number_first,
                    following_number_first,
                    random_new_number,
                ],
            };
            i += 1;
            let new_number: usize = *possible_numbers
                .into_iter()
                .filter(|n| !new_numbers.contains(n))
                .collect::<Vec<usize>>()
                .first()
                .expect("No number fits");
            new_numbers.push(new_number);
        }
        assert!(new_numbers.len() == size);
        assert!(
            (0..size)
                .filter(|n| new_numbers.contains(n))
                .collect::<Vec<usize>>()
                .len()
                == size
        );
        let id = get_id();
        let parent_ids = Some((first.id(), second.id()));
        Self {
            numbers: new_numbers,
            id,
            parent_ids,
        }
    }

    fn mutate(&mut self, problem: &Cities) {
        if 0.01 >= rand::random_range(0.0..1.0) {
            // TODO: mutation rate from settings
            let swap_indices = (0..self.len()).choose_multiple(&mut rand::rng(), 2);
            self.numbers.swap(swap_indices[0], swap_indices[1]);
        }
    }

    fn id(&self) -> Id {
        self.id
    }

    fn parent_ids(&self) -> (Id, Id) {
        self.parent_ids.expect("No parent found")
    }
}

use crate::math::{quantile_from_sorted, transpose};
use crate::settings::{GeneticAlgorithmSettings, PairingSettings, ProblemSettings};
use serde::{Deserialize, Serialize};
use statrs::distribution::{ContinuousCDF, StudentsT};
use std::{
    fs::{File, create_dir_all},
    io::Write,
};

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct DescriptionFlags {
    show_pairing_settings: bool,
    show_genetic_algorithm_settings: bool,
    show_problem_settings: bool,
}

impl DescriptionFlags {
    pub fn from(pairing: bool, genetic_algorithm: bool, problem: bool) -> Self {
        Self {
            show_pairing_settings: pairing,
            show_genetic_algorithm_settings: genetic_algorithm,
            show_problem_settings: problem,
        }
    }
}

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct TValues {
    n: usize,
    m: f64,
    v: f64,
}

impl TValues {
    pub fn from(statistics: Statistics) -> Self {
        let values = statistics.last_n_mean();
        let n: usize = values.len();
        let m: f64 = values.iter().sum::<f64>() / n as f64;
        let v: f64 = values.iter().map(|v| (v - m).powi(2)).sum::<f64>() / (n as f64 - 1.0);
        Self { n, m, v }
    }
}

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct EvaluatedStatistics {
    genetic_algorithm_settings: GeneticAlgorithmSettings,
    problem_settings: ProblemSettings,
    pairing_settings: PairingSettings,
    evaluated_traces: EvaluatedTraces,
    t_values: TValues,
}

impl EvaluatedStatistics {
    pub fn t_test_all(evaluated_statistics: &Vec<Self>) -> Vec<String> {
        let mut results = Vec::new();
        for i in 0..evaluated_statistics.len() {
            for j in (i + 1)..evaluated_statistics.len() {
                let first = &evaluated_statistics[i];
                let second = &evaluated_statistics[j];
                let result = Self::t_test_single(first, second);
                results.extend(result);
            }
        }
        results
    }

    pub fn t_test_single(first: &Self, second: &Self) -> Vec<String> {
        let f = &first.t_values;
        let s = &second.t_values;

        let n1 = f.n as f64;
        let n2 = s.n as f64;

        let m1 = f.m;
        let m2 = s.m;

        let v1 = f.v;
        let v2 = s.v;

        // Welch t statistic
        let t = (m1 - m2) / ((v1 / n1 + v2 / n2).sqrt());

        // Degrees of freedom Welch Satterthwaite
        let df = (v1 / n1 + v2 / n2).powi(2)
            / ((v1 / n1).powi(2) / (n1 - 1.0) + (v2 / n2).powi(2) / (n2 - 1.0));

        // t distribution
        let dist = StudentsT::new(0.0, 1.0, df).expect("Could not create t distribution");

        // two-tailed p
        let p = 2.0 * (1.0 - dist.cdf(t.abs()));
        let p_string = match p < 0.001 {
            true => "<0.001".to_string(),
            false => format!("{:.3}", p),
        };

        let alpha = 0.05;

        let cohens_d_star = (m1 - m2) / (((v1 + v2) / 2.0).sqrt());

        let result = match p <= alpha {
            true => "reject",
            false => "accept",
        };

        let first_name = first.settings_description(&DescriptionFlags::from(true, false, false));
        let second_name = second.settings_description(&DescriptionFlags::from(true, false, false));

        vec![format!(
            "\\({}\\)&\\({}\\)&{:.3}&{:.3}&{:.3}&{:.3}&{}&{}&{:.3}\\\\",
            first_name
                .replace("%", "\\%")
                .replace("<", "\\langle{}")
                .replace(">", "\\rangle{}"),
            second_name
                .replace("%", "\\%")
                .replace("<", "\\langle{}")
                .replace(">", "\\rangle{}"),
            m1,
            m2,
            t,
            df,
            p_string,
            result,
            cohens_d_star.abs(),
        )]
    }

    pub fn from(statistics: Statistics) -> Self {
        let (genetic_algorithm_settings, problem_settings, pairing_settings) =
            statistics.settings();
        let evaluated_traces = EvaluatedTraces::from(statistics.clone());
        let t_values = TValues::from(statistics);
        Self {
            genetic_algorithm_settings,
            problem_settings,
            pairing_settings,
            evaluated_traces,
            t_values,
        }
    }

    pub fn from_string(s: &str) -> Self {
        serde_json::from_str(s).expect("Could not deserialize from string")
    }

    pub fn save(&self, run_id: String, filename: String) {
        let serialized =
            serde_json::to_string_pretty(&self).expect("Could not serialize evaluated statistics");
        create_dir_all(format!(".{}", run_id)).expect("Could not create run id path directory");
        let mut file =
            File::create(format!(".{run_id}/{filename}.json")).expect("Could not create file");
        file.write_all(serialized.as_bytes())
            .expect("Could not write to file");
    }

    pub fn settings_description(&self, description_flags: &DescriptionFlags) -> String {
        let mut descriptions = Vec::new();
        if description_flags.show_pairing_settings {
            descriptions.push(self.pairing_settings.description());
        }
        if description_flags.show_genetic_algorithm_settings {
            descriptions.push(self.genetic_algorithm_settings.description());
        }
        if description_flags.show_problem_settings {
            descriptions.push(self.problem_settings.description());
        }
        descriptions.join(" | ")
    }

    pub fn x_y(&self) -> (Vec<usize>, Vec<f64>) {
        let population_size = self.genetic_algorithm_settings.population_size();
        let y = self.evaluated_traces.average_of_median.clone();
        let x = (0..y.len()).map(|s| s * population_size).collect();
        (x, y)
    }

    // TODO: improve
    pub fn fields(&self) -> Vec<(String, Vec<f64>)> {
        self.evaluated_traces.fields()
    }
}

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
struct EvaluatedTraces {
    // minimum
    min_of_min: Vec<f64>,
    average_of_min: Vec<f64>,
    median_of_min: Vec<f64>,
    // lower quartile
    average_of_lower_quartile: Vec<f64>,
    median_of_lower_quartile: Vec<f64>,
    // median / average
    average_of_average: Vec<f64>,
    average_of_median: Vec<f64>,
    median_of_average: Vec<f64>,
    median_of_median: Vec<f64>,
    // higher quartile
    median_of_higher_quartile: Vec<f64>,
    average_of_higher_quartile: Vec<f64>,
    // max
    median_of_max: Vec<f64>,
    average_of_max: Vec<f64>,
    max_of_max: Vec<f64>,

    step_average_of_max: Vec<f64>,
}

impl EvaluatedTraces {
    pub fn from(statistics: Statistics) -> Self {
        let minimums = statistics.minimums();
        let averages = statistics.averages();
        let maximums = statistics.maximums();
        let (lower_quartiles, medians, higher_quartiles) =
            statistics.lower_quartiles_medians_higher_quartiles();
        // transpose
        let transposed_minimums = transpose(minimums);
        let transposed_averages = transpose(averages);
        let transposed_maximums = transpose(maximums);
        let transposed_lower_quartiles = transpose(lower_quartiles);
        let transposed_medians = transpose(medians);
        let transposed_higher_quartiles = transpose(higher_quartiles);

        let min_of_min = Self::minimum_trace(&transposed_minimums);
        let average_of_min = Self::average_trace(&transposed_minimums);
        let median_of_min = Self::median_trace(&transposed_minimums);
        let average_of_lower_quartile = Self::average_trace(&transposed_lower_quartiles);
        let median_of_lower_quartile = Self::median_trace(&transposed_lower_quartiles);
        let average_of_average = Self::average_trace(&transposed_averages);
        let average_of_median = Self::average_trace(&transposed_medians);
        let median_of_average = Self::median_trace(&transposed_averages);
        let median_of_median = Self::median_trace(&transposed_medians);
        let median_of_higher_quartile = Self::median_trace(&transposed_higher_quartiles);
        let average_of_higher_quartile = Self::average_trace(&transposed_higher_quartiles);
        let median_of_max = Self::median_trace(&transposed_maximums);
        let average_of_max = Self::average_trace(&transposed_maximums);
        let max_of_max = Self::maximum_trace(&transposed_maximums);
        let step_average_of_max = Self::step(average_of_max.clone());

        Self {
            min_of_min,
            average_of_min,
            median_of_min,
            average_of_lower_quartile,
            median_of_lower_quartile,
            average_of_average,
            average_of_median,
            median_of_average,
            median_of_median,
            median_of_higher_quartile,
            average_of_higher_quartile,
            median_of_max,
            average_of_max,
            max_of_max,
            step_average_of_max,
        }
    }

    // TODO: improve
    pub fn fields(&self) -> Vec<(String, Vec<f64>)> {
        vec![
            ("min of min".to_string(), self.min_of_min.clone()),
            ("average of min".to_string(), self.average_of_min.clone()),
            ("median of min".to_string(), self.median_of_min.clone()),
            (
                "average of lower quartile".to_string(),
                self.average_of_lower_quartile.clone(),
            ),
            (
                "median of lower quartile".to_string(),
                self.median_of_lower_quartile.clone(),
            ),
            (
                "average of average".to_string(),
                self.average_of_average.clone(),
            ),
            (
                "average of median".to_string(),
                self.average_of_median.clone(),
            ),
            (
                "median of average".to_string(),
                self.median_of_average.clone(),
            ),
            (
                "median of median".to_string(),
                self.median_of_median.clone(),
            ),
            (
                "median of higher quartile".to_string(),
                self.median_of_higher_quartile.clone(),
            ),
            (
                "average of higher quartile".to_string(),
                self.average_of_higher_quartile.clone(),
            ),
            ("median of max".to_string(), self.median_of_max.clone()),
            ("average of max".to_string(), self.average_of_max.clone()),
            ("max of max".to_string(), self.max_of_max.clone()),
        ]
    }

    fn step(trace: Vec<f64>) -> Vec<f64> {
        let mut new = Vec::new();
        let mut max = f64::MIN;
        for v in trace {
            max = v.max(max);
            new.push(max)
        }
        new
    }

    fn minimum_trace(fitness_by_generations: &Vec<Vec<f64>>) -> Vec<f64> {
        fitness_by_generations
            .iter()
            .reduce(|a, b| if a < b { a } else { b })
            .expect("No minimum trace")
            .clone()
    }

    fn maximum_trace(fitness_by_generations: &Vec<Vec<f64>>) -> Vec<f64> {
        fitness_by_generations
            .iter()
            .reduce(|a, b| if a > b { a } else { b })
            .expect("No maximum trace")
            .clone()
    }

    fn average_trace(fitness_by_generations: &Vec<Vec<f64>>) -> Vec<f64> {
        fitness_by_generations
            .iter()
            .map(|f| f.iter().sum::<f64>() / f.len() as f64)
            .collect()
    }

    fn median_trace(fitness_by_generations: &Vec<Vec<f64>>) -> Vec<f64> {
        fitness_by_generations
            .iter()
            .map(|f| {
                let mut a = f.clone();
                a.sort_by(|a, b| a.partial_cmp(b).expect("Fitness values cannot be None"));
                quantile_from_sorted(f, 0.5)
            })
            .collect()
    }
}

#[derive(PartialEq, Clone)]
pub struct Statistics {
    statistics: Vec<Statistic>,
}

impl Statistics {
    pub fn from(statistics: Vec<Statistic>) -> Self {
        Self { statistics }
    }

    pub fn last_n(&self) -> Vec<Vec<f64>> {
        let mut last = Vec::new();
        for statistic in &self.statistics {
            let length = statistic.generations_fitness.len();
            let last_n: Vec<f64> = statistic
                .generations_fitness
                .iter()
                .map(|f| f.lower_quartile_median_higher_quartile().1)
                .skip(length - 3)
                .collect();
            assert!(last_n.len() == 3, "Incorrect number of last values");
            last.push(last_n);
        }
        last
    }

    pub fn last_n_mean(&self) -> Vec<f64> {
        let last_n = self.last_n();
        let mut last_n_mean = Vec::new();
        for n in last_n {
            let mean = n.iter().sum::<f64>() / n.len() as f64;
            last_n_mean.push(mean);
        }
        last_n_mean
    }

    pub fn settings(&self) -> (GeneticAlgorithmSettings, ProblemSettings, PairingSettings) {
        let first = self.statistics.first().expect("No statistic available");
        let genetic_algorithm_settings = first.genetic_algorithm_settings.clone();
        let problem_settings = first.problem_settings.clone();
        let pairing_settings = first.pairing_settings.clone();
        for statistic in &self.statistics {
            assert!(genetic_algorithm_settings == statistic.genetic_algorithm_settings);
            assert!(problem_settings == statistic.problem_settings);
            assert!(pairing_settings == statistic.pairing_settings);
        }
        (
            genetic_algorithm_settings,
            problem_settings,
            pairing_settings,
        )
    }

    fn minimums(&self) -> Vec<Vec<f64>> {
        self.statistics.iter().map(|s| s.minimums()).collect()
    }

    fn maximums(&self) -> Vec<Vec<f64>> {
        self.statistics.iter().map(|s| s.maximums()).collect()
    }

    fn lower_quartiles_medians_higher_quartiles(
        &self,
    ) -> (Vec<Vec<f64>>, Vec<Vec<f64>>, Vec<Vec<f64>>) {
        let mut lower_quartiles = Vec::new();
        let mut medians = Vec::new();
        let mut higher_quartiles = Vec::new();
        for statistic in &self.statistics {
            let (lower_quartile, median, higher_quartile) =
                statistic.lower_quartiles_medians_higher_quartiles();
            lower_quartiles.push(lower_quartile);
            medians.push(median);
            higher_quartiles.push(higher_quartile);
        }
        (lower_quartiles, medians, higher_quartiles)
    }

    // TODO: remove
    pub fn medians_medians(&self) -> Vec<f64> {
        let mut medians = Vec::new();
        for statistic in &self.statistics {
            let (_, median, _) = statistic.lower_quartiles_medians_higher_quartiles();
            medians.push(median);
        }
        let transposed_medians = transpose(medians);
        transposed_medians
            .iter()
            .map(|m| quantile_from_sorted(m, 0.5))
            .collect()
    }

    pub fn averages(&self) -> Vec<Vec<f64>> {
        self.statistics.iter().map(|s| s.averages()).collect()
    }
    //pub fn quartiles_quartiles(&self) -> _ {
    //}
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct Statistic {
    genetic_algorithm_settings: GeneticAlgorithmSettings,
    problem_settings: ProblemSettings,
    pairing_settings: PairingSettings,
    generations_fitness: Vec<GenerationFitness>,
}

impl Statistic {
    pub fn new(
        genetic_algorithm_settings: GeneticAlgorithmSettings,
        problem_settings: ProblemSettings,
        pairing_settings: PairingSettings,
    ) -> Self {
        let generations_fitness = Vec::new();
        Self {
            genetic_algorithm_settings,
            problem_settings,
            pairing_settings,
            generations_fitness,
        }
    }

    pub fn append_fitness_values(&mut self, fitness_values: Vec<f64>) {
        self.generations_fitness
            .push(GenerationFitness::new(fitness_values));
    }

    pub fn lower_quartiles_medians_higher_quartiles(&self) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let mut lower_quartiles = Vec::new();
        let mut medians = Vec::new();
        let mut higher_quartiles = Vec::new();
        for generations_fitness in &self.generations_fitness {
            let (lower_quartile, median, higher_quartile) =
                generations_fitness.lower_quartile_median_higher_quartile();
            lower_quartiles.push(lower_quartile);
            medians.push(median);
            higher_quartiles.push(higher_quartile);
        }
        (lower_quartiles, medians, higher_quartiles)
    }

    pub fn averages(&self) -> Vec<f64> {
        self.generations_fitness
            .iter()
            .map(|g| g.average())
            .collect()
    }

    pub fn minimums(&self) -> Vec<f64> {
        self.generations_fitness
            .iter()
            .map(|g| g.minimum())
            .collect()
    }

    pub fn maximums(&self) -> Vec<f64> {
        self.generations_fitness
            .iter()
            .map(|g| g.maximum())
            .collect()
    }

    pub fn maximum(&self) -> f64 {
        self.generations_fitness
            .iter()
            .map(|g| g.maximum())
            .reduce(|a, b| if a > b { a } else { b })
            .expect("No statistics data")
    }

    pub fn average(&self) -> f64 {
        self.generations_fitness
            .iter()
            .map(|g| g.average())
            .sum::<f64>()
            / self.generations_fitness.len() as f64
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
struct GenerationFitness {
    fitness_values: Vec<f64>,
}

impl GenerationFitness {
    fn new(fitness_values: Vec<f64>) -> Self {
        Self { fitness_values }
    }

    fn lower_quartile_median_higher_quartile(&self) -> (f64, f64, f64) {
        let mut sorted_fitness = self.fitness_values.clone();
        sorted_fitness.sort_by(|a, b| a.partial_cmp(b).expect("Fitness values cannot be None"));
        let lower_quartile = quantile_from_sorted(&sorted_fitness, 0.25);
        let median = quantile_from_sorted(&sorted_fitness, 0.5);
        let higher_quartile = quantile_from_sorted(&sorted_fitness, 0.75);
        (lower_quartile, median, higher_quartile)
    }

    fn maximum(&self) -> f64 {
        *self
            .fitness_values
            .iter()
            .reduce(|a, b| if a > b { a } else { b })
            .expect("No fitness maximum")
    }

    fn average(&self) -> f64 {
        self.fitness_values.iter().sum::<f64>() / self.fitness_values.len() as f64
    }

    fn minimum(&self) -> f64 {
        *self
            .fitness_values
            .iter()
            .reduce(|a, b| if a < b { a } else { b })
            .expect("No fitness minimum")
    }
}

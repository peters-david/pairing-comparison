mod ga;
mod point;
mod problems;
mod synchronization;

use std::sync::Arc;
use std::time::Duration;

use chrono::Local;
use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use itertools::iproduct;
use rand::Rng;
use threadpool::ThreadPool;

use crate::{
    ga::{
        AntiElitePairing, AsexualPairing, ElitePairing, FitnessProportionatePairing,
        GeneticAlgorithm, Individual, OneRandomPairing, Pairings, Problem, Problems,
        SimilarFitnessPairing, SpatialDistancePairing, ThirdFourthNeighborPairing,
        TwoRandomPairing,
    },
    synchronization::Semaphore,
};
use problems::rofa::Rofa;
use problems::tsp::Cities;
use shared::{
    settings::{GeneticAlgorithmSettings, PairingSettings, ProblemSettings},
    statistics::{EvaluatedStatistics, Statistic, Statistics},
};

fn number_cpus() -> usize {
    let percentage = std::fs::read_to_string("cpu_usage")
        .unwrap_or("100".to_string())
        .trim()
        .parse()
        .unwrap_or(100);
    (num_cpus::get() as f64 * percentage as f64 * 0.01).ceil() as usize
}

fn main() {
    print!("\x1B[2J\x1B[1;1H"); // clear
    let mut pool = ThreadPool::new(number_cpus());
    let run_id = Local::now().format("%Y%m%d%H%M%S").to_string();

    // tsp settings ranges
    let size = (30..=50).step_by(20);

    let mut problem_settings = Vec::new();
    for s in size {
        // problem_settings.push(ProblemSettings::Tsp { size: s });
    }

    // rofa settings ranges
    // TODO: costs can also be parameterized
    let nodes = (20..=20).step_by(50);
    let links_percentage = (30..=30).step_by(40); // the minimum links required are nodes - 1
    let demands_percentage = (50..=50).step_by(40);
    let link_types = (8..=8).step_by(4);

    // here problemsettings are pushed together
    //let mut problem_settings = Vec::new();
    for (n, l_p, d_p, l_t) in iproduct!(nodes, links_percentage, demands_percentage, link_types) {
        problem_settings.push(ProblemSettings::Rofa {
            nodes: n,
            links_percentage: l_p,
            demands_percentage: d_p,
            link_types: l_t,
        });
    }

    let mut problems = Vec::new();
    for problem_setting in problem_settings {
        problems.push({
            match problem_setting {
                p_s @ ProblemSettings::Tsp { size: _ } => Problems::Tsp(Cities::random(&p_s)),
                p_s @ ProblemSettings::Rofa {
                    nodes: _,
                    links_percentage: _,
                    demands_percentage: _,
                    link_types: _,
                } => Problems::Rofa(Rofa::random(&p_s)),
            }
        });
    }

    // genetic algorithm settings
    let population_size_and_generations = vec![(10, 10000), (100, 1000)];
    let survival_rate: Vec<f64> = (3..=7).step_by(2).map(|n| n as f64 * 0.1).collect();
    let mutation_rate = vec![0.01, 0.1];
    let mutation_strength = vec![1];

    let mut genetic_algorithm_settings = Vec::new();
    for ((p, g), s, m_r, m_s) in iproduct!(
        population_size_and_generations,
        survival_rate,
        mutation_rate,
        mutation_strength
    ) {
        genetic_algorithm_settings.push(GeneticAlgorithmSettings::new(p, s, g, m_r, m_s));
    }

    let pairing_settings = vec![
        PairingSettings::AsexualPairing,
        PairingSettings::TwoRandomPairing,
        PairingSettings::OneRandomPairing,
        PairingSettings::NeighborPairing,
        PairingSettings::SimilarFitnessPairing,
        PairingSettings::FitnessProportionatePairing,
        PairingSettings::SpatialDistancePairing {
            desired_individual_distance_percentage: 1,
        },
        PairingSettings::SpatialDistancePairing {
            desired_individual_distance_percentage: 2,
        },
        PairingSettings::SpatialDistancePairing {
            desired_individual_distance_percentage: 3,
        },
        PairingSettings::SpatialDistancePairing {
            desired_individual_distance_percentage: 4,
        },
        PairingSettings::SpatialDistancePairing {
            desired_individual_distance_percentage: 5,
        },
        PairingSettings::SpatialDistancePairing {
            desired_individual_distance_percentage: 6,
        },
        PairingSettings::SpatialDistancePairing {
            desired_individual_distance_percentage: 7,
        },
        PairingSettings::SpatialDistancePairing {
            desired_individual_distance_percentage: 8,
        },
        PairingSettings::SpatialDistancePairing {
            desired_individual_distance_percentage: 9,
        },
        PairingSettings::SpatialDistancePairing {
            desired_individual_distance_percentage: 10,
        },
        PairingSettings::SpatialDistancePairing {
            desired_individual_distance_percentage: 12,
        },
        PairingSettings::SpatialDistancePairing {
            desired_individual_distance_percentage: 15,
        },
        PairingSettings::SpatialDistancePairing {
            desired_individual_distance_percentage: 20,
        },
        PairingSettings::SpatialDistancePairing {
            desired_individual_distance_percentage: 30,
        },
        PairingSettings::SpatialDistancePairing {
            desired_individual_distance_percentage: 40,
        },
        PairingSettings::SpatialDistancePairing {
            desired_individual_distance_percentage: 50,
        },
        PairingSettings::SpatialDistancePairing {
            desired_individual_distance_percentage: 70,
        },
        PairingSettings::SpatialDistancePairing {
            desired_individual_distance_percentage: 90,
        },
        PairingSettings::SpatialDistancePairing {
            desired_individual_distance_percentage: 100,
        },
    ];

    let m = Arc::new(MultiProgress::with_draw_target(
        indicatif::ProgressDrawTarget::stderr_with_hz(8),
    ));
    let style = ProgressStyle::with_template("{msg:<12} [{bar:100.cyan/blue}] {pos}/{len}")
        .expect("Could not create progress bar style");

    let iterations: usize = 10;
    let total_executions =
        genetic_algorithm_settings.len() * pairing_settings.len() * problems.len();
    let overall_progress_bar = m.add(ProgressBar::new(total_executions as u64));
    overall_progress_bar.set_style(style.clone());
    overall_progress_bar.set_message("Overall");

    let max_queued_tasks = number_cpus() + 1;
    let semaphore = Arc::new(Semaphore::new(max_queued_tasks));

    for (unique_number, (g_a_s, p_s, p)) in
        iproduct!(genetic_algorithm_settings, pairing_settings, problems).enumerate()
    {
        semaphore.acquire();
        let semaphore = Arc::clone(&semaphore);
        let run_id = run_id.clone();
        let overall_progress_bar_handle = overall_progress_bar.clone();
        let m = m.clone();
        pool.set_num_threads(number_cpus());
        pool.execute(move || {
            let progress_bar = m.add(ProgressBar::new(iterations as u64));
            progress_bar.set_style(
                ProgressStyle::with_template("{msg:<12} [{bar:100.cyan/blue}] {pos}/{len}")
                    .expect("Could not create progress bar style"),
            );
            progress_bar.set_message(format!("{unique_number}"));

            let mut statistics = Vec::new();
            for _i in 0..iterations {
                let p = p.clone();
                let p_s = p_s.clone();
                let g_a_s = g_a_s.clone();
                let statistic = match p {
                    Problems::Tsp(p) => create_pairing_run_genetic_algorithm(p, p_s, g_a_s),
                    Problems::Rofa(p) => create_pairing_run_genetic_algorithm(p, p_s, g_a_s),
                };
                statistics.push(statistic);
                progress_bar.inc(1);
            }
            progress_bar.finish_and_clear();

            let statistics = Statistics::from(statistics);
            let evaluated_statistics = EvaluatedStatistics::from(statistics);
            evaluated_statistics.save(run_id, unique_number.to_string());

            overall_progress_bar_handle.inc(1);
            semaphore.release();
        });
    }
    pool.join();
    overall_progress_bar.finish();
}

fn create_pairing_run_genetic_algorithm(
    problem: impl Problem,
    pairing_settings: PairingSettings,
    genetic_algorithm_settings: GeneticAlgorithmSettings,
) -> Statistic {
    let pairing = pairing_from_pairing_settings(pairing_settings);
    let mut ga = GeneticAlgorithm::new(problem, pairing, genetic_algorithm_settings);
    ga.run();
    ga.get_statistic()
}

fn pairing_from_pairing_settings<I: Individual>(pairing_settings: PairingSettings) -> Pairings<I> {
    match pairing_settings {
        p @ PairingSettings::AsexualPairing => Pairings::AsexualPairing(AsexualPairing::new(p)),
        p @ PairingSettings::TwoRandomPairing => {
            Pairings::TwoRandomPairing(TwoRandomPairing::new(p))
        }
        p @ PairingSettings::OneRandomPairing => {
            Pairings::OneRandomPairing(OneRandomPairing::new(p))
        }
        p @ PairingSettings::SimilarFitnessPairing => {
            Pairings::SimilarFitnessPairing(SimilarFitnessPairing::new(p))
        }
        p @ PairingSettings::NeighborPairing => {
            Pairings::ThirdFourthNeighborPairing(ThirdFourthNeighborPairing::new(p))
        }
        p @ PairingSettings::FitnessProportionatePairing => {
            Pairings::FitnessProportionatePairing(FitnessProportionatePairing::new(p))
        }
        p @ PairingSettings::ElitePairing => Pairings::ElitePairing(ElitePairing::new(p)),
        p @ PairingSettings::AntiElitePairing => {
            Pairings::AntiElitePairing(AntiElitePairing::new(p))
        }
        p @ PairingSettings::SpatialDistancePairing {
            desired_individual_distance_percentage,
        } => Pairings::SpatialDistancePairing(SpatialDistancePairing::new(p)),
    }
}

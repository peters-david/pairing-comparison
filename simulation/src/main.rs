mod ga;
mod point;
mod problems;
mod synchronization;

use std::{
    fs::{read_dir, remove_dir_all},
    path::Path,
    sync::Arc,
};

use chrono::Local;
use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use itertools::{iproduct, Itertools};
use rand::{rngs::StdRng, SeedableRng};
use threadpool::ThreadPool;

use crate::{
    ga::{
        AsexualPairing, GeneticAlgorithm, Individual, Pairings, Problem, Problems, RandomPairing,
        SimilarFitnessPairing, SpatialDistancePairing,
    },
    synchronization::Semaphore,
};
use problems::rofa::Rofa;
use problems::tsp::Cities;
use shared::{
    settings::{GeneticAlgorithmSettings, IndividualQuantity, PairingSettings, ProblemSettings},
    statistics::{EvaluatedStatistics, Statistic, Statistics},
};

#[derive(Parser)]
struct Args {
    #[arg(long)]
    resume: bool,
}

fn main() {
    one();
}

fn one() {
    // rofa settings ranges
    // TODO: costs can also be parameterized
    let nodes: Vec<usize> = (30..=30).step_by(10).collect();
    let links_percentage: Vec<usize> = (30..=30).step_by(30).collect(); // the minimum links required are nodes - 1
    let demands_percentage: Vec<usize> = (70..=70).step_by(30).collect();
    let link_types: Vec<usize> = (6..=12).step_by(8).collect();
    let cfa_settings = (nodes, links_percentage, demands_percentage, link_types);

    // genetic algorithm settings
    let population_size_and_generations = vec![(200, 5000)];
    let survival_rate: Vec<f64> = (5..=5).step_by(4).map(|n| n as f64 * 0.1).collect();
    let mutation_rate = vec![0.005];
    let mutation_strength = vec![1];
    let ga_settings = (
        population_size_and_generations,
        survival_rate,
        mutation_rate,
        mutation_strength,
    );

    // layer 1
    let mut individual_quantities = vec![
        IndividualQuantity::Random,
        IndividualQuantity::FitnessProportionate,
    ];
    for p in [5] {
        individual_quantities.extend(vec![
            IndividualQuantity::Elite { percentage: p },
            IndividualQuantity::AntiElite { percentage: p },
        ]);
    }

    //layer 2
    let mut pairing_settings = Vec::new();
    for i_q in individual_quantities {
        pairing_settings.extend(vec![
            // PairingSettings::AsexualPairing { quantity: i_q },
            PairingSettings::RandomPairing { quantity: i_q },
        ]);
        // for s in [3] {
        //     pairing_settings.push(PairingSettings::SimilarFitnessPairing {
        //         quantity: i_q,
        //         similarity: s,
        //     });
        // }
        // for d_i_d_p in [3] {
        //     pairing_settings.push(PairingSettings::SpatialDistancePairing {
        //         quantity: i_q,
        //         desired_individual_distance_percentage: d_i_d_p,
        //     });
        // }
    }

    run(
        &"onexxxxxxxxxxx".to_string(),
        cfa_settings,
        ga_settings,
        pairing_settings,
    );
}

fn run(
    run_id: &String,
    cfa_settings: (Vec<usize>, Vec<usize>, Vec<usize>, Vec<usize>),
    ga_settings: (Vec<(usize, usize)>, Vec<f64>, Vec<f64>, Vec<usize>),
    pairing_settings: Vec<PairingSettings>,
) {
    // print!("\x1B[2J\x1B[1;1H"); // clear
    println!("{}", run_id);
    let mut pool = ThreadPool::new(number_cpus());
    let mut first_level_rng = StdRng::seed_from_u64(0u64);

    let args = Args::parse();
    let existing_unique_numbers = match args.resume {
        true => {
            let existing_unique_numbers = existing_unique_numbers(&run_id);
            existing_unique_numbers
        }
        false => {
            if Path::new(&format!(".{}", run_id)).exists() {
                remove_dir_all(format!(".{}", run_id)).expect("Could not remove previous run");
                println!("Removed previous run");
            }
            let existing_unique_numbers = vec![];
            existing_unique_numbers
        }
    };

    let (nodes, links_percentage, demands_percentage, link_types) = cfa_settings;
    let mut problem_settings = Vec::new();
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
                p_s @ ProblemSettings::Tsp { size: _ } => {
                    Problems::Tsp(Cities::random(&mut first_level_rng, &p_s))
                }
                p_s @ ProblemSettings::Rofa {
                    nodes: _,
                    links_percentage: _,
                    demands_percentage: _,
                    link_types: _,
                } => Problems::Rofa(Rofa::random(&mut first_level_rng, &p_s)),
            }
        });
    }

    let (population_size_and_generations, survival_rate, mutation_rate, mutation_strength) =
        ga_settings;
    let mut genetic_algorithm_settings = Vec::new();
    for ((p, g), s, m_r, m_s) in iproduct!(
        population_size_and_generations,
        survival_rate,
        mutation_rate,
        mutation_strength
    ) {
        genetic_algorithm_settings.push(GeneticAlgorithmSettings::new(p, s, g, m_r, m_s));
    }

    let m = Arc::new(MultiProgress::with_draw_target(
        indicatif::ProgressDrawTarget::stderr_with_hz(8),
    ));
    let style = ProgressStyle::with_template("{msg:<15} [{bar:100.cyan/blue}] {pos}/{len} ({eta})")
        .expect("Could not create progress bar style");

    let iterations: usize = 100;
    let total_executions =
        genetic_algorithm_settings.len() * pairing_settings.len() * problems.len()
            - existing_unique_numbers.len();
    let overall_progress_bar = m.add(ProgressBar::new(total_executions as u64));
    overall_progress_bar.set_style(style.clone());
    overall_progress_bar.set_message("Overall");

    let max_queued_tasks = number_cpus() + 2;
    let semaphore = Arc::new(Semaphore::new(max_queued_tasks));

    for (unique_number, (g_a_s, p_s, p)) in
        iproduct!(genetic_algorithm_settings, pairing_settings, problems).enumerate()
    {
        semaphore.acquire();
        let semaphore = Arc::clone(&semaphore);
        let run_id = run_id.clone();
        let ran_previously = existing_unique_numbers.contains(&unique_number);
        let overall_progress_bar_handle = overall_progress_bar.clone();
        let m = m.clone();
        pool.set_num_threads(number_cpus());
        if ran_previously {
            semaphore.release();
        } else {
            pool.execute(move || {
                let progress_bar = m.add(ProgressBar::new(iterations as u64));
                progress_bar.set_style(
                    ProgressStyle::with_template(
                        "{msg:<15} [{bar:100.cyan/blue}] {pos}/{len} ({eta})",
                    )
                    .expect("Could not create progress bar style"),
                );
                progress_bar.set_message(format!("{unique_number}"));

                let mut rng = StdRng::seed_from_u64(unique_number as u64);
                let mut statistics = Vec::new();
                for _i in 0..iterations {
                    let p = p.clone();
                    let p_s = p_s.clone();
                    let g_a_s = g_a_s.clone();
                    let p_b = progress_bar.clone();
                    let statistic = match p {
                        Problems::Tsp(p) => create_pairing_run_genetic_algorithm(
                            &mut rng,
                            p,
                            p_s,
                            g_a_s,
                            unique_number,
                            p_b,
                        ),
                        Problems::Rofa(p) => create_pairing_run_genetic_algorithm(
                            &mut rng,
                            p,
                            p_s,
                            g_a_s,
                            unique_number,
                            p_b,
                        ),
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
    }
    pool.join();
    overall_progress_bar.finish();
}

fn create_pairing_run_genetic_algorithm(
    rng: &mut StdRng,
    problem: impl Problem,
    pairing_settings: PairingSettings,
    genetic_algorithm_settings: GeneticAlgorithmSettings,
    unique_number: usize,
    progress_bar: ProgressBar,
) -> Statistic {
    let pairing = pairing_from_pairing_settings(pairing_settings);
    let mut ga = GeneticAlgorithm::new(problem, pairing, genetic_algorithm_settings);
    ga.run(rng, unique_number, progress_bar);
    ga.get_statistic()
}

fn pairing_from_pairing_settings<I: Individual>(pairing_settings: PairingSettings) -> Pairings<I> {
    match pairing_settings {
        p @ PairingSettings::AsexualPairing { .. } => {
            Pairings::AsexualPairing(AsexualPairing::new(p))
        }
        p @ PairingSettings::RandomPairing { .. } => Pairings::RandomPairing(RandomPairing::new(p)),
        p @ PairingSettings::SimilarFitnessPairing { .. } => {
            Pairings::SimilarFitnessPairing(SimilarFitnessPairing::new(p))
        }
        p @ PairingSettings::SpatialDistancePairing { .. } => {
            Pairings::SpatialDistancePairing(SpatialDistancePairing::new(p))
        }
    }
}

fn number_cpus() -> usize {
    let percentage = std::fs::read_to_string("cpu_usage")
        .unwrap_or("100".to_string())
        .trim()
        .parse()
        .unwrap_or(100);
    (num_cpus::get() as f64 * percentage as f64 * 0.01).ceil() as usize
}

fn last_run() -> String {
    read_dir(".")
        .expect("Could not get run id directory")
        .map(|e| {
            e.expect("Could not get directory in run id directory")
                .path()
        })
        .filter(|p| p.is_dir())
        .filter_map(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .filter(|n| n.starts_with(".20"))
                .map(|_| p.clone())
        })
        .max_by(|a, b| {
            a.file_name()
                .expect("Error getting filename")
                .cmp(b.file_name().expect("Error getting filename"))
        })
        .expect("Coud not get highest directory in run id directory")
        .file_name()
        .expect("Could not get name of highest run id directory")
        .to_str()
        .expect("Could not turn run id directory name into string")[1..]
        .to_string()
}

fn existing_unique_numbers(run_id: &String) -> Vec<usize> {
    read_dir(format!(".{}", run_id))
        .expect("Could not get run id directory")
        .map(|e| {
            e.expect("Could not get file in run id directory")
                .file_name()
                .to_str()
                .expect("Could not turn filename into string")
                .strip_suffix(".json")
                .expect("Could not strip suffix")
                .parse::<usize>()
                .expect("Could not parse number from filename")
        })
        .collect()
}

mod ga;
mod point;
mod problems;
mod synchronization;

use std::{
    fs::{read_dir, remove_dir_all, write},
    path::Path,
    sync::Arc,
};

use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use itertools::iproduct;
use rand::{rngs::StdRng, SeedableRng};
use threadpool::ThreadPool;

use crate::{
    ga::{
        AsexualPairing, GeneticAlgorithm, Individual, Pairings, Problem, Problems, RandomPairing,
        SimilarFitnessPairing, SpatialDistancePairing,
    },
    problems::rofa::RoutingAndCapacityPlan,
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
    e1();
    e2();
    e3();
    e4();
    e5a();
    e5b();
    e6a();
    e6b();
    e7();
    e8();
    e10();
    e11();
}

fn e1() {
    let cfa_settings = (vec![25], vec![30], vec![20], vec![15]);
    let ga_settings = (vec![(100, 2000)], vec![0.5], vec![0.01], vec![1]);
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

    let mut pairing_settings = Vec::new();
    for i_q in individual_quantities {
        pairing_settings.extend(vec![PairingSettings::RandomPairing { quantity: i_q }]);
    }

    run(
        "e1".to_string(),
        cfa_settings,
        ga_settings,
        pairing_settings,
    );
}

fn e2() {
    let cfa_settings = (vec![25], vec![30], vec![20], vec![15]);
    let ga_settings = (vec![(100, 2000)], vec![0.5], vec![0.01], vec![1]);
    let mut individual_quantities = vec![IndividualQuantity::Random];

    let mut pairing_settings = Vec::new();
    for i_q in individual_quantities {
        pairing_settings.extend(vec![
            PairingSettings::AsexualPairing { quantity: i_q },
            PairingSettings::RandomPairing { quantity: i_q },
        ]);
        for s in [100] {
            pairing_settings.push(PairingSettings::SimilarFitnessPairing {
                quantity: i_q,
                similarity: s,
            });
        }
        for d_i_d_p in [10] {
            pairing_settings.push(PairingSettings::SpatialDistancePairing {
                quantity: i_q,
                desired_individual_distance_percentage: d_i_d_p,
            });
        }
    }

    run(
        "e2".to_string(),
        cfa_settings,
        ga_settings,
        pairing_settings,
    );
}

fn e3() {
    let cfa_settings = (vec![25], vec![30], vec![20], vec![15]);
    let ga_settings = (vec![(100, 2000)], vec![0.5], vec![0.01], vec![1]);
    let mut individual_quantities = vec![IndividualQuantity::Random];

    let mut pairing_settings = Vec::new();
    for i_q in individual_quantities {
        pairing_settings.extend(vec![]);
        for d_i_d_p in (10..=100).step_by(10) {
            pairing_settings.push(PairingSettings::SpatialDistancePairing {
                quantity: i_q,
                desired_individual_distance_percentage: d_i_d_p,
            });
        }
    }

    run(
        "e3".to_string(),
        cfa_settings,
        ga_settings,
        pairing_settings,
    );
}

fn e4() {
    let cfa_settings = (vec![25], vec![30], vec![20], vec![15]);
    let ga_settings = (vec![(100, 2000)], vec![0.5], vec![0.01], vec![1]);
    let mut individual_quantities = vec![IndividualQuantity::Random];

    let mut pairing_settings = Vec::new();
    for i_q in individual_quantities {
        pairing_settings.extend(vec![]);
        for s in (10..=100).step_by(10) {
            pairing_settings.push(PairingSettings::SimilarFitnessPairing {
                quantity: i_q,
                similarity: s,
            });
        }
    }

    run(
        "e4".to_string(),
        cfa_settings,
        ga_settings,
        pairing_settings,
    );
}

fn e5a() {
    let cfa_settings = (vec![25], vec![30], vec![20], vec![15]);
    let ga_settings = (vec![(100, 3000)], vec![0.9], vec![0.01], vec![1]);
    let mut individual_quantities = vec![IndividualQuantity::Random];

    let mut pairing_settings = Vec::new();
    for i_q in individual_quantities {
        pairing_settings.extend(vec![
            PairingSettings::AsexualPairing { quantity: i_q },
            PairingSettings::RandomPairing { quantity: i_q },
        ]);
        for s in (100..=100).step_by(10) {
            pairing_settings.push(PairingSettings::SimilarFitnessPairing {
                quantity: i_q,
                similarity: s,
            });
        }
        for d_i_d_p in (10..=10).step_by(10) {
            pairing_settings.push(PairingSettings::SpatialDistancePairing {
                quantity: i_q,
                desired_individual_distance_percentage: d_i_d_p,
            });
        }
    }

    run(
        "e5a".to_string(),
        cfa_settings,
        ga_settings,
        pairing_settings,
    );
}

fn e5b() {
    let cfa_settings = (vec![25], vec![30], vec![20], vec![15]);
    let ga_settings = (vec![(100, 3000)], vec![0.1], vec![0.01], vec![1]);
    let mut individual_quantities = vec![IndividualQuantity::Random];

    let mut pairing_settings = Vec::new();
    for i_q in individual_quantities {
        pairing_settings.extend(vec![
            PairingSettings::AsexualPairing { quantity: i_q },
            PairingSettings::RandomPairing { quantity: i_q },
        ]);
        for s in (100..=100).step_by(10) {
            pairing_settings.push(PairingSettings::SimilarFitnessPairing {
                quantity: i_q,
                similarity: s,
            });
        }
        for d_i_d_p in (10..=10).step_by(10) {
            pairing_settings.push(PairingSettings::SpatialDistancePairing {
                quantity: i_q,
                desired_individual_distance_percentage: d_i_d_p,
            });
        }
    }

    run(
        "e5b".to_string(),
        cfa_settings,
        ga_settings,
        pairing_settings,
    );
}

fn e6a() {
    let cfa_settings = (vec![10], vec![30], vec![20], vec![15]);
    let ga_settings = (vec![(100, 2000)], vec![0.5], vec![0.01], vec![1]);
    let mut individual_quantities = vec![IndividualQuantity::Random];

    let mut pairing_settings = Vec::new();
    for i_q in individual_quantities {
        pairing_settings.extend(vec![
            PairingSettings::AsexualPairing { quantity: i_q },
            PairingSettings::RandomPairing { quantity: i_q },
        ]);
        for s in (100..=100).step_by(10) {
            pairing_settings.push(PairingSettings::SimilarFitnessPairing {
                quantity: i_q,
                similarity: s,
            });
        }
        for d_i_d_p in (10..=10).step_by(10) {
            pairing_settings.push(PairingSettings::SpatialDistancePairing {
                quantity: i_q,
                desired_individual_distance_percentage: d_i_d_p,
            });
        }
    }

    run(
        "e6a".to_string(),
        cfa_settings,
        ga_settings,
        pairing_settings,
    );
}

fn e6b() {
    let cfa_settings = (vec![50], vec![30], vec![20], vec![15]);
    let ga_settings = (vec![(100, 2000)], vec![0.5], vec![0.01], vec![1]);
    let mut individual_quantities = vec![IndividualQuantity::Random];

    let mut pairing_settings = Vec::new();
    for i_q in individual_quantities {
        pairing_settings.extend(vec![
            PairingSettings::AsexualPairing { quantity: i_q },
            PairingSettings::RandomPairing { quantity: i_q },
        ]);
        for s in (100..=100).step_by(10) {
            pairing_settings.push(PairingSettings::SimilarFitnessPairing {
                quantity: i_q,
                similarity: s,
            });
        }
        for d_i_d_p in (10..=10).step_by(10) {
            pairing_settings.push(PairingSettings::SpatialDistancePairing {
                quantity: i_q,
                desired_individual_distance_percentage: d_i_d_p,
            });
        }
    }

    run(
        "e6b".to_string(),
        cfa_settings,
        ga_settings,
        pairing_settings,
    );
}

fn e7() {
    let cfa_settings = (vec![100], vec![30], vec![20], vec![15]);
    let ga_settings = (vec![(50, 100)], vec![0.5], vec![0.01], vec![1]);
    let mut individual_quantities = vec![IndividualQuantity::Random];

    let mut pairing_settings = Vec::new();
    for i_q in individual_quantities {
        pairing_settings.extend(vec![
            PairingSettings::AsexualPairing { quantity: i_q },
            PairingSettings::RandomPairing { quantity: i_q },
        ]);
        for s in (100..=100).step_by(10) {
            pairing_settings.push(PairingSettings::SimilarFitnessPairing {
                quantity: i_q,
                similarity: s,
            });
        }
        for d_i_d_p in (60..=60).step_by(10) {
            pairing_settings.push(PairingSettings::SpatialDistancePairing {
                quantity: i_q,
                desired_individual_distance_percentage: d_i_d_p,
            });
        }
    }

    run(
        "e7".to_string(),
        cfa_settings,
        ga_settings,
        pairing_settings,
    );
}

fn e8() {
    let cfa_settings = (vec![4], vec![100], vec![100], vec![3]);
    let ga_settings = (vec![(20, 100)], vec![0.5], vec![0.01], vec![1]);
    let mut individual_quantities = vec![IndividualQuantity::Random];

    let mut pairing_settings = Vec::new();
    for i_q in individual_quantities {
        pairing_settings.extend(vec![
            PairingSettings::AsexualPairing { quantity: i_q },
            PairingSettings::RandomPairing { quantity: i_q },
        ]);
        for s in (100..=100).step_by(10) {
            pairing_settings.push(PairingSettings::SimilarFitnessPairing {
                quantity: i_q,
                similarity: s,
            });
        }
        for d_i_d_p in (60..=60).step_by(10) {
            pairing_settings.push(PairingSettings::SpatialDistancePairing {
                quantity: i_q,
                desired_individual_distance_percentage: d_i_d_p,
            });
        }
    }

    run(
        "e8a".to_string(),
        cfa_settings,
        ga_settings,
        pairing_settings,
    );
    exhaustive_search(&"e8b".to_string(), (4, 100, 100, 3));
}

fn e9() {
    let cfa_settings = (vec![25], vec![30], vec![20], vec![15]);
    let ga_settings = (vec![(100, 2000)], vec![0.5], vec![0.01], vec![1]);
    let mut individual_quantities = vec![];
    for p in [100] {
        individual_quantities.extend(vec![
            IndividualQuantity::Elite { percentage: p },
            IndividualQuantity::AntiElite { percentage: p },
        ]);
    }

    let mut pairing_settings = Vec::new();
    for i_q in individual_quantities {
        pairing_settings.extend(vec![
            PairingSettings::AsexualPairing { quantity: i_q },
            PairingSettings::RandomPairing { quantity: i_q },
        ]);
        for s in (100..=100).step_by(40) {
            pairing_settings.push(PairingSettings::SimilarFitnessPairing {
                quantity: i_q,
                similarity: s,
            });
        }
        for d_i_d_p in (10..=10).step_by(40) {
            pairing_settings.push(PairingSettings::SpatialDistancePairing {
                quantity: i_q,
                desired_individual_distance_percentage: d_i_d_p,
            });
        }
    }

    run(
        "e9".to_string(),
        cfa_settings,
        ga_settings,
        pairing_settings,
    );
}

fn e10() {
    let cfa_settings = (vec![25], vec![30], vec![20], vec![15]);
    let ga_settings = (vec![(1, 200000)], vec![1.0], vec![1.0], vec![100]);
    let mut individual_quantities = vec![];
    for p in [100] {
        individual_quantities.extend(vec![IndividualQuantity::Elite { percentage: p }]);
    }

    let mut pairing_settings = Vec::new();
    for i_q in individual_quantities {
        pairing_settings.extend(vec![PairingSettings::AsexualPairing { quantity: i_q }]);
    }

    run(
        "e10".to_string(),
        cfa_settings,
        ga_settings,
        pairing_settings,
    );
}

fn e11() {
    let cfa_settings = (vec![25], vec![30], vec![20], vec![15]);
    let ga_settings = (vec![(2, 200000)], vec![0.4], vec![0.1], vec![2]);
    let mut individual_quantities = vec![];
    for p in [100] {
        individual_quantities.extend(vec![IndividualQuantity::Elite { percentage: p }]);
    }

    let mut pairing_settings = Vec::new();
    for i_q in individual_quantities {
        pairing_settings.extend(vec![PairingSettings::AsexualPairing { quantity: i_q }]);
    }

    run(
        "e11".to_string(),
        cfa_settings,
        ga_settings,
        pairing_settings,
    );
}

fn run(
    mut run_id: String,
    cfa_settings: (Vec<usize>, Vec<usize>, Vec<usize>, Vec<usize>),
    ga_settings: (Vec<(usize, usize)>, Vec<f64>, Vec<f64>, Vec<usize>),
    pairing_settings: Vec<PairingSettings>,
) {
    while run_id.len() < 14 {
        run_id.push('x');
    }
    assert!(run_id.len() == 14);
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
    // uncomment to disable progress bar
    // m.set_draw_target(ProgressDrawTarget::hidden());
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
                        "{msg:<15} [{bar:100.cyan/blue}] {pos}/{len} running for {elapsed_precise}",
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

fn exhaustive_search(run_id: &String, cfa_setting: (usize, usize, usize, usize)) {
    let (nodes, links_percentage, demands_percentage, link_types) = cfa_setting;
    let problem_settings = ProblemSettings::Rofa {
        nodes,
        links_percentage,
        demands_percentage,
        link_types,
    };
    let mut rng = StdRng::seed_from_u64(0u64);
    let problem = Rofa::random(&mut rng, &problem_settings);
    let links = problem.number_links();
    let demands = problem.number_demands();
    let solutions = RoutingAndCapacityPlan::exhaustive(problem.clone());
    let (length, best) = solutions.map(|s| -problem.cost(&s)).fold(
        (0usize, f64::MIN),
        |(count, current_max), item| {
            let new_max = current_max.max(item);
            (count + 1, new_max)
        },
    );
    println!(
        "length:{},nodes:{},links:{},link_types:{},demands:{}",
        length, nodes, links, link_types, demands
    );
    write(
        format!("{}.txt", run_id),
        format!(
            "length:{},nodes:{},links:{},demands:{},link_types:{}\nbest:{}",
            length, nodes, links, demands, link_types, best
        ),
    )
    .expect("Cannot write to file");
}

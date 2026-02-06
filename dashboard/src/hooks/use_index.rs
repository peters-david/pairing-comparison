use crate::index_reference::ResultFiles;

use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use yew::{UseStateHandle, hook, use_effect_with, use_state};

#[hook]
pub fn use_multiple() -> (
    UseStateHandle<Option<ResultFiles>>,
    UseStateHandle<Option<Vec<String>>>,
    UseStateHandle<Option<ResultFiles>>,
) {
    let index = use_state(|| None::<ResultFiles>);
    let runs = use_state(|| None::<Vec<String>>);
    let latest_run = use_state(|| None::<ResultFiles>);

    {
        let index = index.clone();
        let runs = runs.clone();
        let latest_run = latest_run.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                let result_files = all_result_files().await;
                let mut run_ids = result_files.runs();
                run_ids.sort();
                runs.set(Some(run_ids));
                println!("{}", result_files.runs()[0]);
                latest_run.set(Some(result_files.only_latest_run()));
                index.set(Some(result_files));
            });
            || ()
        });
    }

    (index, runs, latest_run)
}

#[hook]
pub fn use_index() -> UseStateHandle<Option<ResultFiles>> {
    let index = use_state(|| None::<ResultFiles>);

    {
        let index = index.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                let result_files = all_result_files().await;
                index.set(Some(result_files));
            });
            || ()
        });
    }
    index
}

#[hook]
pub fn use_runs() -> UseStateHandle<Option<Vec<String>>> {
    let runs = use_state(|| None::<Vec<String>>);

    {
        let runs = runs.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                let result_files = all_result_files().await;
                let mut run_ids = result_files.runs();
                run_ids.sort();
                runs.set(Some(run_ids));
            });
            || ()
        });
    }

    runs
}

#[hook]
pub fn use_latest_run() -> UseStateHandle<Option<ResultFiles>> {
    let latest_run = use_state(|| None::<ResultFiles>);

    {
        let latest_run = latest_run.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                let result_files = all_result_files().await;
                latest_run.set(Some(result_files.only_latest_run()));
            });
            || ()
        });
    }

    latest_run
}

async fn get_index() -> Vec<String> {
    Request::get("/results/index.json")
        .send()
        .await
        .expect("Could not load file")
        .json()
        .await
        .expect("Could not parse json")
}

pub async fn all_result_files() -> ResultFiles {
    let index_paths = get_index().await;
    ResultFiles::from(index_paths)
}

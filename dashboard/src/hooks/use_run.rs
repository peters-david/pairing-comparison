use crate::hooks::use_index::{use_index, use_multiple};
use crate::index_reference::ResultFiles;
use shared::statistics::{EvaluatedStatistics, Statistics};
use wasm_bindgen_futures::spawn_local;
use yew::{UseStateHandle, hook, use_effect_with, use_state};

#[hook]
pub fn use_run_result_files(run_id: String) -> UseStateHandle<Option<ResultFiles>> {
    use crate::index_reference::ResultFiles;

    let run = use_state(|| None::<Vec<EvaluatedStatistics>>);
    let run_result_files = use_state(|| None::<ResultFiles>);
    let index = use_index();
    {
        let index = index.clone();
        let run_id = run_id.clone();
        let run_result_files = run_result_files.clone();
        use_effect_with((*index).clone(), move |index| {
            if let Some(i) = &*index {
                let i = i.clone();
                spawn_local(async move {
                    let desired_run = i.filter_by_run(&run_id);
                    run_result_files.set(Some(desired_run));
                    // let statistics = desired_run.load().await;
                    // run.set(Some(statistics));
                });
            }
            || ()
        });
    }
    // run
    run_result_files
}

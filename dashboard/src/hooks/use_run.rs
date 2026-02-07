use crate::hooks::use_index::use_index;
use crate::index_reference::ResultFiles;
use yew::{UseStateHandle, hook, use_effect_with, use_state};

#[hook]
pub fn use_run_result_files(run_id: String) -> UseStateHandle<Option<ResultFiles>> {
    let run_result_files = use_state(|| None::<ResultFiles>);
    let index = use_index();
    {
        let index = index.clone();
        let run_result_files = run_result_files.clone();
        use_effect_with((run_id.clone(), index.clone()), move |_| {
            if let Some(i) = &*index {
                let i = i.clone();
                let desired_run = i.filter_by_run(&run_id);
                run_result_files.set(Some(desired_run));
            }
            || ()
        });
    }
    run_result_files
}

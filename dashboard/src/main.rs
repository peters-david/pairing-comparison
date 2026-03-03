mod components;
mod hooks;
mod index_reference;
mod models;
mod utils;
mod wasm;

use shared::statistics::DescriptionFlags;
use yew::prelude::*;

use crate::{
    components::{
        plot::{ResultFilesPlot, StaticPlot},
        plot_with_settings::PlotWithSettings,
        single_plot_settings::SinglePlotSettingsComponent,
    },
    hooks::use_index::{use_index, use_multiple, use_runs},
    utils::console::console_error,
};

#[function_component(App)]
fn app() -> Html {
    let runs = use_runs();
    let selected_run = use_state(|| None::<String>);

    let on_change_selected_run = {
        let selected_run = selected_run.clone();
        Callback::from(move |e: Event| {
            let input = e.target_dyn_into::<web_sys::HtmlSelectElement>();
            if let Some(s) = input {
                selected_run.set(Some(s.value()));
            }
        })
    };

    html! {
        <div>
            <h1>{"Dashboard"}</h1>
            <h3>{"E1"}</h3>
            <StaticPlot run_id={".e1xxxxxxxxxxxx".to_string()} description_flags={DescriptionFlags::from(true, false, false)} />
            <h3>{"E2"}</h3>
            <StaticPlot run_id={".e2xxxxxxxxxxxx".to_string()} description_flags={DescriptionFlags::from(true, false, false)} />
            <h3>{"E3"}</h3>
            <StaticPlot run_id={".e3xxxxxxxxxxxx".to_string()} description_flags={DescriptionFlags::from(true, false, false)} />
            <h3>{"E4"}</h3>
            <StaticPlot run_id={".e4xxxxxxxxxxxx".to_string()} description_flags={DescriptionFlags::from(true, false, false)} />
            <h3>{"E5a"}</h3>
            <StaticPlot run_id={".e5axxxxxxxxxxx".to_string()} description_flags={DescriptionFlags::from(true, false, false)} />
            <h3>{"E5b"}</h3>
            <StaticPlot run_id={".e5bxxxxxxxxxxx".to_string()} description_flags={DescriptionFlags::from(true, false, false)} />
            <h3>{"E6a"}</h3>
            <StaticPlot run_id={".e6axxxxxxxxxxx".to_string()} description_flags={DescriptionFlags::from(true, false, false)} />
            <h3>{"E6b"}</h3>
            <StaticPlot run_id={".e6bxxxxxxxxxxx".to_string()} description_flags={DescriptionFlags::from(true, false, false)} />
            <h3>{"E7"}</h3>
            <StaticPlot run_id={".e7xxxxxxxxxxxx".to_string()} description_flags={DescriptionFlags::from(true, false, false)} />
            <h3>{"E8a (b is exhaustive)"}</h3>
            <StaticPlot run_id={".e8axxxxxxxxxxx".to_string()} description_flags={DescriptionFlags::from(true, false, false)} />
            <h3>{"E9 is table"}</h3>
            <h3>{"E10"}</h3>
            <StaticPlot run_id={".e10xxxxxxxxxxx".to_string()} description_flags={DescriptionFlags::from(true, false, false)} />
            <h3>{"E11"}</h3>
            <StaticPlot run_id={".e11xxxxxxxxxxx".to_string()} description_flags={DescriptionFlags::from(true, false, false)} />
            {
                if let Some(r) = &*runs {
                    html! {
                        <select value={(*selected_run).clone()} onchange={on_change_selected_run}>
                            {
                                r.iter().map(|run| {
                                    html! {
                                        <option value={run.clone()}>
                                            {run}
                                        </option>
                                    }
                                }).collect::<Html>()
                            }
                            {
                                (selected_run.is_none()).then(||  {
                                    html! { <option value="">{"Select run..."}</option> }
                                })
                            }
                        </select>
                    }
                } else {
                    html! {
                        <h3>{"Runs not loaded yet"}</h3>
                    }
                }
            }
            if let Some(s_r) = (*selected_run).clone() {
                <PlotWithSettings selected_run={s_r.clone()}/>
            }
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}

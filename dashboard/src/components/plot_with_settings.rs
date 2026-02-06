use shared::statistics::EvaluatedStatistics;
use wasm_bindgen_futures::spawn_local;
use yew::{
    Callback, Html, MouseEvent, Properties, function_component, html, use_effect_with, use_state,
};

use crate::{
    components::{
        plot::{ResultFilesPlot, StatisticsPlot},
        single_plot_settings::SinglePlotSettingsComponent,
    },
    hooks::{
        use_index::{use_index, use_latest_run, use_multiple},
        use_run::use_run_result_files,
    },
    models::plot_settings::PlotSettings,
};

#[derive(Properties, PartialEq)]
pub struct PlotWithSettingsProps {
    pub selected_run: String,
}

#[function_component(PlotWithSettings)]
pub fn plot_with_settings(props: &PlotWithSettingsProps) -> Html {
    let run = use_run_result_files(props.selected_run.clone());
    let plot_settings = use_state(|| None::<PlotSettings>);
    let filtered_evaluated_statistics = use_state(|| None::<Vec<EvaluatedStatistics>>);
    let plot_out_of_sync = use_state(|| false);

    {
        let plot_settings = plot_settings.clone();
        use_effect_with((*run).clone(), |run| {
            if let Some(r) = (*run).clone() {
                spawn_local(async move {
                    let loaded_plot_settings =
                        PlotSettings::from_result_files_content(r.load_strings().await);
                    plot_settings.set(Some(loaded_plot_settings));
                });
            }
            || ()
        });
    }
    {
        let plot_settings = plot_settings.clone();
        let plot_out_of_sync = plot_out_of_sync.clone();
        use_effect_with(plot_settings.clone(), move |_| {
            plot_out_of_sync.set(true);
            || ()
        });
    }

    let on_change_plot_settings = {
        let plot_settings = plot_settings.clone();
        Callback::from(move |new_plot_settings| {
            plot_settings.set(Some(new_plot_settings));
        })
    };

    let update_plot = {
        let plot_settings = plot_settings.clone();
        let run = run.clone();
        let filtered_evaluated_statistics = filtered_evaluated_statistics.clone();
        let plot_out_of_sync = plot_out_of_sync.clone();
        Callback::from(move |_e: MouseEvent| {
            if let Some(p_s) = (*plot_settings).clone() {
                let filtered_evaluated_statistics = filtered_evaluated_statistics.clone();
                let plot_out_of_sync = plot_out_of_sync.clone();
                let run = run.clone();
                spawn_local(async move {
                    if let Some(r) = (*run).clone() {
                        let new = r.load_by_settings(p_s).await;
                        filtered_evaluated_statistics.set(Some(new));
                        plot_out_of_sync.set(false);
                    }
                });
            }
        })
    };

    html! {
        <div>
            {
                if let Some(f_e_s) = &*filtered_evaluated_statistics {
                    html! {
                        <StatisticsPlot all_statistics={f_e_s.clone()} />
                    }
                } else {
                    html! {
                        <h1>{"No statistics yet"}</h1>
                    }
                }
            }
            {
                if let Some(p_s) = &*plot_settings {
                    html! {
                        <>
                            {
                                if *plot_out_of_sync {
                                    html!{
                                        <>
                                            <h1>{"Plot is not reflecting selected settings"}</h1>
                                            <button onclick={update_plot}>{"Update plot"}</button>
                                        </>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                            <SinglePlotSettingsComponent plot_settings={p_s.clone()} update_plot_settings={on_change_plot_settings} />
                        </>
                    }
                } else {
                    html! {
                        <h1>{"No plot settings yet"}</h1>
                    }
                }
            }
        </div>
    }
}

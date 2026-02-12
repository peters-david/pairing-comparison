use std::collections::HashSet;

use yew::{Callback, Html, Properties, function_component, html, use_state};

use crate::models::plot_settings::PlotSettings;

#[derive(Properties, PartialEq, Clone)]
pub struct SinglePlotSettingsComponentProps {
    pub plot_settings: PlotSettings,
    pub update_plot_settings: Callback<PlotSettings>,
}
#[function_component(SinglePlotSettingsComponent)]
pub fn single_plot_settings_component(props: &SinglePlotSettingsComponentProps) -> Html {
    let SinglePlotSettingsComponentProps {
        plot_settings,
        update_plot_settings,
    } = props.clone();

    let toggle_variation = {
        let plot_settings = plot_settings.clone();
        Callback::from(move |path: Vec<String>| {
            let mut new = plot_settings.clone();
            new.toggle(path);
            update_plot_settings.emit(new);
        })
    };

    html! {
        <div>
            {
                html!{
                    <SinglePlotSettingsSubcomponent name={"Settings".to_string()} paths={plot_settings.paths()} on_toggle={toggle_variation.clone()} />
                }
            }
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
struct SinglePlotSettingsSubcomponentProps {
    name: String,
    paths: Vec<Vec<String>>,
    on_toggle: Callback<Vec<String>>,
}

#[function_component(SinglePlotSettingsSubcomponent)]
fn single_plot_settings_subcomponent(props: &SinglePlotSettingsSubcomponentProps) -> Html {
    let SinglePlotSettingsSubcomponentProps {
        name,
        paths,
        on_toggle,
    } = props.clone();

    let mut seen = HashSet::new();
    for path in paths {}

    html! {}
}

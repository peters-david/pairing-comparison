use std::collections::HashSet;

use yew::{Callback, Html, Properties, function_component, html, use_state};

use crate::{models::plot_settings::PlotSettings, utils::console::console_error};

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
                    <SinglePlotSettingsSubcomponent paths={plot_settings.paths()} on_toggle={toggle_variation.clone()} />
                }
            }
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
struct SinglePlotSettingsSubcomponentProps {
    paths: Vec<Vec<String>>,
    on_toggle: Callback<Vec<String>>,
}

#[function_component(SinglePlotSettingsSubcomponent)]
fn single_plot_settings_subcomponent(props: &SinglePlotSettingsSubcomponentProps) -> Html {
    let SinglePlotSettingsSubcomponentProps { paths, on_toggle } = props.clone();

    console_error(&format!("{:#?}", paths));
    let mut first_paths = HashSet::new();
    for path in &paths {
        if !path.is_empty() {
            let first_path = first_path_part(path);
            first_paths.insert(first_path.clone());
        }
    }
    let mut parts = Vec::new();
    for first_path in first_paths {
        let subpaths: Vec<Vec<String>> = paths
            .clone()
            .into_iter()
            .filter(|p| !p.is_empty() && first_path_part(p) == first_path)
            .map(|mut p| {
                p.remove(0);
                p
            })
            .collect();
        parts.push(html! {
            <div style="display: inline-block;">
                <label style="margin-right: 2px; margin-bottom: 2px;">
                    <input type="checkbox" checked={checked} onclick={Callback::from(move |_| {
                        on_toggle.emit(path.clone());
                    })} />
                    {name.clone()}
                </label>
            </div>
            //
            <div style="padding-left:20px;">
            <button/>
            <p>{first_path}</p>
            <SinglePlotSettingsSubcomponent paths={subpaths} on_toggle={on_toggle.clone()} />
            </div>
        });
    }

    html! { <div> { for parts } </div> }
}

fn first_path_part(path: &Vec<String>) -> String {
    path.first().expect("Cannot get first path part").clone()
}

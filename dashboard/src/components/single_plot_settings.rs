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
                    <SinglePlotSettingsSubcomponent current_path={vec![]} plot_settings={plot_settings} on_toggle={toggle_variation.clone()} />
                }
            }
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
struct SinglePlotSettingsSubcomponentProps {
    current_path: Vec<String>,
    plot_settings: PlotSettings,
    on_toggle: Callback<Vec<String>>,
}

#[function_component(SinglePlotSettingsSubcomponent)]
fn single_plot_settings_subcomponent(props: &SinglePlotSettingsSubcomponentProps) -> Html {
    let SinglePlotSettingsSubcomponentProps {
        current_path,
        plot_settings,
        on_toggle,
    } = props.clone();

    console_error(&format!("{:#?}", current_path));
    let first_paths = plot_settings.get_first_paths(&current_path);
    let mut parts = Vec::new();
    for first_path in first_paths {
        let mut current_subpath = current_path.clone();
        current_subpath.push(first_path.clone());

        if plot_settings.get_first_paths(&current_subpath).is_empty() {
            let on_toggle = on_toggle.clone();
            parts.push(html! {
                <div style="display: inline-block;">
                    <label style="margin: 0; padding: 0;">
                        <input type="checkbox" checked={plot_settings.is_checked(&current_subpath)} onclick={Callback::from(move |_| {
                            on_toggle.emit(current_subpath.clone());
                        })} />
                        {first_path.clone()}
                    </label>
                </div>
            });
        } else {
            parts.push(html! {
                <div>
                    <p style="margin: 0; padding: 0;">{first_path}</p>
                    <SinglePlotSettingsSubcomponent current_path={current_subpath} plot_settings={plot_settings.clone()} on_toggle={on_toggle.clone()} />
                </div>
            });
        }
    }

    html! { <div style="padding-left:10px; margin: 0;"> { for parts } </div> }
}

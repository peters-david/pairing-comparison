use yew::{Callback, Html, Properties, function_component, html, use_state};

use crate::models::plot_settings::{PlotSettings, Sub};

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
            if let Some(first) = path.first() {
                if let Some(sub) = new.settings_map.get_mut(first) {
                    toggle_value_at_path(sub, &path[1..]);
                }
            }
            update_plot_settings.emit(new);
        })
    };
    html! {
        <div>
            {
                html!{
                    { for plot_settings.settings_map.iter().map(|(k, v)| {
                        html! {
                            <SinglePlotSettingsSubcomponent name={k.clone()} sub={v.clone()} path={vec![k.clone()]} on_toggle={toggle_variation.clone()} />
                        }
                    })}
                }
            }
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
struct SinglePlotSettingsSubcomponentProps {
    name: String,
    sub: Sub,
    path: Vec<String>,
    on_toggle: Callback<Vec<String>>,
}

#[function_component(SinglePlotSettingsSubcomponent)]
fn single_plot_settings_subcomponent(props: &SinglePlotSettingsSubcomponentProps) -> Html {
    let SinglePlotSettingsSubcomponentProps {
        name,
        sub,
        path,
        on_toggle,
    } = props.clone();

    match sub {
        Sub::SubSettings(subsettings) => {
            html! {
                <div style="margin-left: 10px; margin-bottom: 5px">
                    {name.clone()}
                    { for subsettings.subsettings.iter().map(|(k, v)| {
                        let mut child_path = path.clone();
                        child_path.push(k.clone());
                        html! {
                            <SinglePlotSettingsSubcomponent name={k.clone()} sub={v.clone()} path={child_path} on_toggle={on_toggle.clone()} />
                        }
                    })}
                </div>
            }
        }
        Sub::Variations(variations) => {
            html! {
                <div style="margin-left: 10px; margin-bottom: 5px">
                    {name.clone()}
                    <br />
                    { for variations.variations.iter().map(|(name, value)| {
                        let mut path = path.clone();
                        path.push(name.clone());
                        let checked = *value;
                        let on_toggle = on_toggle.clone();
                        html! {
                            <div style="display: inline-block;">
                                <label style="margin-right: 2px; margin-bottom: 2px;">
                                    <input type="checkbox" checked={checked} onclick={Callback::from(move |_| {
                                        on_toggle.emit(path.clone());
                                    })} />
                                    {name.clone()}
                                </label>
                            </div>
                        }
                    })}
                </div>
            }
        }
    }
}

fn toggle_value_at_path(sub: &mut Sub, path: &[String]) {
    if path.is_empty() {
        return;
    }
    match sub {
        Sub::SubSettings(subsettings) => {
            if let Some(next) = subsettings.subsettings.get_mut(&path[0]) {
                toggle_value_at_path(next, &path[1..]);
            }
        }
        Sub::Variations(variations) => {
            if path.len() == 1
                && let Some(variation) = variations.variations.get_mut(&path[0])
            {
                *variation = !*variation;
            }
        }
    }
}

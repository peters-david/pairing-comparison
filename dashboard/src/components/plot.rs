use crate::{index_reference::ResultFiles, utils::console::console_error, wasm::plotly::newPlot};
use serde::Serialize;
use serde_wasm_bindgen::to_value;
use shared::statistics::{EvaluatedStatistics, Statistics};
use wasm_bindgen_futures::spawn_local;
use yew::{
    Html, Properties, function_component, html,
    suspense::{use_future, use_future_with},
    use_effect, use_effect_with, use_state,
};

#[derive(Properties, PartialEq, Clone)]
pub struct ResultFilesPlotProps {
    pub result_files: ResultFiles,
}

#[function_component(ResultFilesPlot)]
pub fn result_files_plot(props: &ResultFilesPlotProps) -> Html {
    let statistics = use_state(|| None::<Vec<EvaluatedStatistics>>);
    {
        let result_files = props.result_files.clone();
        let statistics = statistics.clone();
        use_future(|| async move {
            statistics.set(Some(result_files.load().await));
        });
    }
    html! {
        if let Some(s) = &*statistics {
            <StatisticsPlot all_statistics={s.clone()}/>
        } else {
            <h3>{"No data in result files plot"}</h3>
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct StatisticsPlotProps {
    pub all_statistics: Vec<EvaluatedStatistics>,
}

#[function_component(StatisticsPlot)]
pub fn statistics_plot(props: &StatisticsPlotProps) -> Html {
    let data = use_state(|| None::<Vec<Trace>>);
    let layout = Layout {
        title: "Trace".to_string(),
        width: 2500,
        height: 1100,
    };
    {
        let data = data.clone();
        use_effect_with(props.all_statistics.clone(), move |statistics| {
            console_error(&format!("{:#?}", statistics));
            let mut traces = statistics
                .iter()
                .map(|t| {
                    // let (name_lower, data_lower) = t.fields()[3].clone();
                    // let trace_lower = Trace {
                    //     x: (0..data_lower.len()).collect(),
                    //     y: data_lower,
                    //     name: name_lower,
                    //     r#type: "scatter".to_string(),
                    // };
                    // let (name_upper, data_upper) = t.fields()[10].clone();
                    // let trace_upper = Trace {
                    //     x: (0..data_upper.len()).collect(),
                    //     y: data_upper,
                    //     name: name_upper,
                    //     r#type: "scatter".to_string(),
                    // };
                    let description = t.settings_description();
                    let (x, y) = t.x_y();
                    let trace_median = Trace {
                        x,
                        y,
                        name: description,
                        r#type: "scatter".to_string(),
                    };
                    // let (name, data) = t.fields()[12].clone();
                    // let trace_max = Trace {
                    //     x: (0..data.len()).collect(),
                    //     y: data,
                    //     name: "max: ".to_string() + &description,
                    //     r#type: "scatter".to_string(),
                    // };
                    vec![trace_median]
                })
                .collect::<Vec<Vec<Trace>>>()
                .iter()
                .flatten()
                .cloned()
                .collect::<Vec<Trace>>();
            traces.sort_by(|a, b| {
                b.y.last()
                    .expect("No y value in trace")
                    .partial_cmp(a.y.last().expect("No y value in trace"))
                    .expect("Could not order traces by end result")
            });
            data.set(Some(traces));
            || ()
        });
    }
    html! {
        if let Some(d) = (*data).clone() {
            <TracePlot data={d} layout={layout} />
        } else {
            <h1>{"No data in statistics plot"}</h1>
        }
    }
}

#[derive(Clone, Serialize, PartialEq)]
struct Trace {
    x: Vec<usize>,
    y: Vec<f64>,
    name: String,
    r#type: String,
}

impl Trace {
    fn multiple_from_xs(xs: Vec<Statistics>) -> Vec<Self> {
        todo!()
    }
}

#[derive(Clone, Serialize, PartialEq)]
struct Layout {
    title: String,
    width: usize,
    height: usize,
}

#[derive(Properties, PartialEq)]
struct TracePlotProps {
    data: Vec<Trace>,
    layout: Layout,
}

#[function_component(TracePlot)]
fn trace_plot(props: &TracePlotProps) -> Html {
    {
        let data = props.data.clone();
        let layout = props.layout.clone();
        use_effect_with(data.clone(), move |data| {
            let plot_data = to_value(&data).expect("Cannot turn trace plot data into js value");
            let layout_data = to_value(&layout).expect("Cannot turn plot layout into js value");

            newPlot("plot", &plot_data, &layout_data);
            || {}
        });
    }

    html! {
        <div id="plot"></div>
    }
}

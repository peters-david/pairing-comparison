use crate::{
    hooks::use_run::use_run_result_files, index_reference::ResultFiles,
    utils::console::console_error, wasm::plotly::newPlot,
};
use serde::Serialize;
use serde_wasm_bindgen::to_value;
use shared::statistics::{DescriptionFlags, EvaluatedStatistics, Statistics};
use wasm_bindgen_futures::spawn_local;
use yew::{
    Html, Properties, function_component, html,
    suspense::{use_future, use_future_with},
    use_effect, use_effect_with, use_state,
};

#[derive(Properties, PartialEq, Clone)]
pub struct ResultFilesPlotProps {
    pub name: String,
    pub result_files: ResultFiles,
    pub description_flags: DescriptionFlags,
}

#[function_component(ResultFilesPlot)]
pub fn result_files_plot(props: &ResultFilesPlotProps) -> Html {
    let statistics = use_state(|| None::<Vec<EvaluatedStatistics>>);
    {
        let result_files = props.result_files.clone();
        let statistics = statistics.clone();
        use_future(|| async move {
            console_error(&format!("len:{}", result_files.len()));
            statistics.set(Some(result_files.load().await));
        });
    }
    html! {
        if let Some(s) = &*statistics {
            <StatisticsPlot name={props.name.clone()} all_statistics={s.clone()} description_flags={props.description_flags.clone()}/>
            {
                for EvaluatedStatistics::t_test_all(s).iter().map(|i| html!{
                    <>
                        {i}
                        <br/>
                    </>
                })
            }
            <h3>{"-------------------"}</h3>
        } else {
            <h3>{"No data in result files plot"}</h3>
        }
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct StaticPlotProps {
    pub run_id: String,
    pub description_flags: DescriptionFlags,
}

#[function_component(StaticPlot)]
pub fn static_plot(props: &StaticPlotProps) -> Html {
    let run = use_run_result_files(props.run_id.clone());
    html! {
        if let Some(r_f) = &*run {
            <ResultFilesPlot name={props.run_id.clone()} result_files={r_f.clone()} description_flags={props.description_flags.clone()}/>
        } else {
            {"Plot still loading"}
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct StatisticsPlotProps {
    pub name: String,
    pub all_statistics: Vec<EvaluatedStatistics>,
    pub description_flags: DescriptionFlags,
}

#[function_component(StatisticsPlot)]
pub fn statistics_plot(props: &StatisticsPlotProps) -> Html {
    let data = use_state(|| None::<Vec<Trace>>);
    let layout = Layout {
        title: None,
        width: 1200,
        height: 600,
        margins: Margins {
            l: 0,
            r: 0,
            t: 0,
            b: 0,
            pad: 0,
        },
        autosize: false,
        xaxis: Axis {
            showline: true,
            title: Title {
                text: "Evaluations".to_string(),
                font: Size { size: 22 },
                standoff: 0,
            },
            tickfont: Size { size: 16 },
            automargin: true,
            domain: (0.0, 1.0),
        },
        yaxis: Axis {
            showline: true,
            title: Title {
                text: "Fitness".to_string(),
                font: Size { size: 22 },
                standoff: 0,
            },
            tickfont: Size { size: 16 },
            automargin: true,
            domain: (0.0, 1.0),
        },
        legend: Font {
            font: Size { size: 16 },
        },
    };
    {
        let data = data.clone();
        let description_flags = props.description_flags.clone();
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
                    let description = t.settings_description(&description_flags);
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
            <TracePlot name={props.name.clone()} data={d} layout={layout} />
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

#[derive(Clone, Serialize, PartialEq)]
struct Margins {
    l: usize,
    r: usize,
    t: usize,
    b: usize,
    pad: usize,
}

#[derive(Clone, Serialize, PartialEq)]
struct Mode {
    mode: bool,
}

#[derive(Clone, Serialize, PartialEq)]
struct Layout {
    title: Option<String>,
    margins: Margins,
    autosize: bool,
    xaxis: Axis,
    yaxis: Axis,
    legend: Font,
    width: usize,
    height: usize,
}

#[derive(Clone, Serialize, PartialEq)]
struct Font {
    font: Size,
}

#[derive(Clone, Serialize, PartialEq)]
struct Size {
    size: usize,
}

#[derive(Clone, Serialize, PartialEq)]
struct Title {
    text: String,
    font: Size,
    standoff: usize,
}

#[derive(Clone, Serialize, PartialEq)]
struct Axis {
    showline: bool,
    title: Title,
    tickfont: Size,
    automargin: bool,
    domain: (f64, f64),
}

#[derive(Clone, Serialize, PartialEq)]
struct Config {
    responsive: bool,
    toImageButtonOptions: ImageOptions,
}

#[derive(Clone, Serialize, PartialEq)]
struct ImageOptions {
    format: String,
    filename: String,
    width: usize,
    height: usize,
    scale: usize,
}

#[derive(Properties, PartialEq)]
struct TracePlotProps {
    name: String,
    data: Vec<Trace>,
    layout: Layout,
}

#[function_component(TracePlot)]
fn trace_plot(props: &TracePlotProps) -> Html {
    let config = Config {
        responsive: true,
        toImageButtonOptions: ImageOptions {
            format: "svg".to_string(),
            filename: "plot".to_string(),
            width: 1000,
            height: 500,
            scale: 1,
        },
    };
    {
        let data = props.data.clone();
        let layout = props.layout.clone();
        let name = props.name.clone();
        use_effect_with(data.clone(), move |data| {
            let plot_data = to_value(&data).expect("Cannot turn trace plot data into js value");
            let layout_data = to_value(&layout).expect("Cannot turn plot layout into js value");
            let config_data = to_value(&config).expect("Connot turn export config into js value");

            newPlot(&name, &plot_data, &layout_data, &config_data);
            || {}
        });
    }

    html! {
        <div id={props.name.clone()} style={"padding: 0; margin: 0;"}></div>
    }
}

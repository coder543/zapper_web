#![recursion_limit = "128"]

#[macro_use]
extern crate zapper;
use zapper::{ast, compile, optimizer, tokenizer, Bytecode};

#[macro_use]
extern crate yew;
use yew::prelude::*;

extern crate stdweb;
use stdweb::web::Date;

use std::fmt::Write;
use std::io::Write as IOWrite;
use std::{str, f64};

pub trait ReadableDuration {
    fn readable(&self) -> String;
}

impl ReadableDuration for f64 {
    fn readable(&self) -> String {
        let time = self / 1000.0;
        if time < 1.0 {
            format!("{:.0} ms", time * 1000.0)
        } else {
            format!("{:.0} secs", time)
        }
    }
}

type Context = ();

#[derive(Clone, Debug, PartialEq)]
enum OutputMode {
    Rendered,
    UnoptAST,
    OptAST,
    Bytecode,
}

struct Model {
    template: String,
    output: Vec<u8>,
    error: String,
    stats: String,
    output_mode: OutputMode,
    group: Vec<Person>,
}

impl Model {
    fn render(&mut self) -> Result<(), String> {
        self.error.clear();
        self.output.clear();
        self.stats.clear();
        let start = Date::now();

        let env = Provider {
            provider: "john doe".to_string(),
            provider_code: 31,
        };

        match self.output_mode {
            OutputMode::Rendered => {
                let mut bytecode = compile(&self.template, &env)?;

                for person in &self.group {
                    bytecode.render(person, &mut self.output).unwrap();
                }

                if self.output.len() > 500000 {
                    self.output.truncate(500000);
                    write!(
                        self.output,
                        "... truncated to 500,000 characters to keep browser from becoming sluggish."
                    ).unwrap();
                }

                let time = Date::now() - start;
                let rows_per_second = if time > 0.0 {
                    self.group.len() as f64 / (time / 1000.0)
                } else {
                    f64::INFINITY
                };

                write!(
                    &mut self.stats,
                    "render took {}, approx. {:.0} rows per second",
                    time.readable(),
                    rows_per_second
                ).unwrap();
            }
            OutputMode::UnoptAST => {
                let tokenizer = tokenizer::Tokenizer::new(&self.template);
                let ast = ast::parse(tokenizer)?;
                write!(&mut self.output, "{:#?}", ast).unwrap();
            }
            OutputMode::OptAST => {
                let tokenizer = tokenizer::Tokenizer::new(&self.template);
                let ast = ast::parse(tokenizer)?;
                let ast = optimizer::optimize(ast, &env);
                write!(&mut self.output, "{:#?}", ast).unwrap();
            }
            OutputMode::Bytecode => {
                let tokenizer = tokenizer::Tokenizer::new(&self.template);
                let ast = ast::parse(tokenizer)?;
                let ast = optimizer::optimize(ast, &env);
                write!(&mut self.output, "{:#?}", Bytecode::from_ast(ast, &env)?).unwrap();
            }
        }

        Ok(())
    }

    fn gen_group(&mut self, row_count: u64) {
        self.group.clear();
        for i in 0..row_count {
            self.group.push(Person {
                id: 1 + i,
                name: "Bob".to_string(),
                age: 49,
                weight: 170.3 + i as f64,
            });
        }
    }
}

#[derive(Clone, Debug)]
enum Msg {
    Input(String),
    ChangeRowCount(String),
    ChangeMode(OutputMode),
}

impl Component<Context> for Model {
    // Some details omitted. Explore the examples to get more.

    type Msg = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: &mut Env<Context, Self>) -> Self {
        let mut model = Model {
            template: String::from("{{provider}} {{provider_code + 4}} {{id}} {{name | toupper}} {{age | sqrt}} {{weight / 2.2 | round 2}}kg\n"),
            output: Vec::new(),
            error: String::new(),
            stats: String::new(),
            output_mode: OutputMode::Rendered,
            group: Vec::new(),
        };
        model.gen_group(10000);
        match model.render() {
            Ok(bc) => bc,
            Err(err) => {
                model.error = format!("error compiling template: {}", err);
            }
        };
        model
    }

    fn update(&mut self, msg: Self::Msg, _: &mut Env<Context, Self>) -> ShouldRender {
        match msg {
            Msg::Input(input) => {
                self.template = input;
            }
            Msg::ChangeMode(mode) => {
                self.output_mode = mode;
            }
            Msg::ChangeRowCount(row_count) => {
                let row_count = match row_count.parse::<u64>() {
                    Ok(v) => v,
                    Err(_) => return false,
                };
                self.gen_group(row_count);
            }
        };

        match self.render() {
            Ok(bc) => bc,
            Err(err) => {
                self.error = format!("error compiling template: {}", err);
            }
        };

        true
    }
}

impl Renderable<Context, Model> for Model {
    fn view(&self) -> Html<Context, Self> {
        html! {
            <div style="width: 100%; max-width: 840px; padding: 5px; margin: auto;",>

                <h2>{ "WASM Zapper Demo" }</h2>

                <input type="number",
                    id="rowcount",
                    style="width: 80px",
                    oninput=|e: InputData| Msg::ChangeRowCount(e.value),
                    value=self.group.len(),
                    />
                <label for="rowcount", style="margin-right: 10px",>{ " Rows " }</label>

                <input type="radio",
                    id="rendered",
                    name="outputmode",
                    checked=self.output_mode == OutputMode::Rendered,
                    oninput=|_e: InputData| Msg::ChangeMode(OutputMode::Rendered),
                    onclick=|_e: MouseData| Msg::ChangeMode(OutputMode::Rendered),
                    />
                <label for="rendered",>{ "Rendered " }</label>

                <input type="radio",
                    id="unoptast",
                    name="outputmode",
                    checked=self.output_mode == OutputMode::UnoptAST,
                    oninput=|_e: InputData| Msg::ChangeMode(OutputMode::UnoptAST),
                    onclick=|_e: MouseData| Msg::ChangeMode(OutputMode::UnoptAST),
                    />
                <label for="unoptast",>{ "Unoptimized AST " }</label>

                <input type="radio",
                    id="optast",
                    name="outputmode",
                    checked=self.output_mode == OutputMode::OptAST,
                    oninput=|_e: InputData| Msg::ChangeMode(OutputMode::OptAST),
                    onclick=|_e: MouseData| Msg::ChangeMode(OutputMode::OptAST),
                    />
                <label for="optast",>{ "Optimized AST " }</label>

                <input type="radio",
                    id="bytecode",
                    name="outputmode",
                    checked=self.output_mode == OutputMode::Bytecode,
                    oninput=|_e: InputData| Msg::ChangeMode(OutputMode::Bytecode),
                    onclick=|_e: MouseData| Msg::ChangeMode(OutputMode::Bytecode),
                    />
                <label for="bytecode",>{ "Bytecode" }</label><br/><br/>

                <textarea rows=5,
                    value=&self.template,
                    oninput=|e: InputData| Msg::Input(e.value),
                    style="width: 98%",
                    placeholder="Enter a Zapper template here",>
                </textarea>

                <div style="width: 100%; color: #333",>
                    { &self.stats }
                </div>

                <div style="width: 100%; color: red",>
                    { &self.error }
                </div>

                <pre style="width: 100%; background-color: #eee",>
                    { unsafe { str::from_utf8_unchecked(&self.output) } }
                </pre>

            </div>
        }
    }
}

#[derive(ZapperRunner)]
#[filter = "sqrt/0n"]
#[filter = "round/1n"]
#[filter = "toupper/0s"]
struct Person {
    id: u64,
    name: String,
    age: u32,
    weight: f64,
}

#[derive(ZapperEnv)]
#[runner = "Person"]
struct Provider {
    provider: String,
    provider_code: u32,
}

fn sqrt(_data: &Person, _args: &[f64], input: f64) -> f64 {
    input.sqrt()
}

fn round(_data: &Person, args: &[f64], input: f64) -> f64 {
    let digits = args[0];
    let factor = 10u32.pow(digits as u32) as f64;
    let value = (input * factor).round() as f64;
    value / factor
}

fn toupper(_data: &Person, _args: &[f64], input: &str, buffer: &mut String) {
    for c in input.as_bytes() {
        buffer.push(c.to_ascii_uppercase() as char)
    }
}

fn main() {
    yew::initialize();
    let app: App<_, Model> = App::new(());
    app.mount_to_body();
    yew::run_loop();
}

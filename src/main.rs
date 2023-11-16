use yew::prelude::*;
use yew_icons::{Icon, IconId};
use std::collections::HashMap;

mod teanga;
mod serialization;

#[derive(Clone, PartialEq, Properties)]
pub struct Layer {
    name: String,
    selected: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct DocumentViewProps {
    pub meta: HashMap<String, teanga::LayerDesc>,
    pub document: teanga::Document,
    pub layers: Vec<Layer>,
    pub on_next_doc: Callback<String>,
    pub on_prev_doc: Callback<String>,
}

#[function_component]
fn DocumentView(props : &DocumentViewProps) -> Html {
    let on_next_doc = props.on_next_doc.clone();
    let on_prev_doc = props.on_prev_doc.clone();
    html! {
        <div class="p-4 flex flex-row h-full">
            <div class="basis-1">
                <button class="button h-full" onclick={move |_| on_prev_doc.emit("".to_string())}><Icon icon_id={IconId::BootstrapChevronCompactLeft}/></button>
            </div>
            <div class="grow">
                <h2 class="text-xl font-bold">{ "Document" }</h2>
                {{
                    let text_layers = props.document.get_text_layers();
                    props.document.get_annos(&props.meta);
                    if text_layers.len() == 1 {
                        html! {
                            <div>{ text_layers.iter().next().unwrap().1 }</div>
                        }
                    } else {
                        text_layers.iter().map(|(layer_name, layer)| {
                            html! {
                                <div>
                                    <h3 class="font-semibold">{ layer_name }</h3>
                                    <div>{ layer }</div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                }}
                //<div><ruby class="border-green-900 border-2 rounded-md">{"Lorem"}<rt class="bg-green-900 text-white border-2 border-green-900 rounded-t-md">{"dolor"}</rt></ruby> {" ipsum dolor sit amet, consectetur adipiscing elit. Nulla euismod, nisl vitae ultrices ultricies, nunc nisl ultricies nunc, vitae ultricies nisl nisl eget nisl. Donec euismod, nisl vitae ultrices ultricies, nunc nisl ultricies nunc, vitae ultricies nisl nisl eget nisl. Donec euismod, nisl vitae ultrices ultricies, nunc nisl ultricies nunc, vitae ultricies nisl nisl eget nisl." }</div>
            </div>
            <div class="basis-1">
                <button class="button h-full" onclick={move |_| on_next_doc.emit("".to_string())}><Icon icon_id={IconId::BootstrapChevronCompactRight}/></button>
            </div>
        </div>
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct LayerSelectProps {
    pub layers: Vec<Layer>,
    pub on_layer_enable: Callback<usize>,
}

pub struct LayerSelect;

impl Component for LayerSelect {
    type Message = ();
    type Properties = LayerSelectProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx : &Context<Self>) -> Html {
        let layers = ctx.props().layers.clone();
        let on_layer_enable = ctx.props().on_layer_enable.clone();
        html! {
            <div class="p-4">
                <h3 class="font-semibold mb-4">{ "Layers" }</h3>
                <ul class="text-sm font-medium bg-bwhite border border-gray-400 rounded-md">
                {
                    {
                        let mut layer_html = Vec::new();

                    for (i, layer) in layers.iter().enumerate() {
                        let on_layer_enable = on_layer_enable.clone();
                        layer_html.push(html! {
                            <li class="w-full border-b border-gray-400 rounded-t-lg">
                                <div class="flex items-center flex-row ps-3">
                                <input type="checkbox" checked={layer.selected} class="w-4 h-4 border-gray-500 rounded"
                                onclick={move |_| on_layer_enable.emit(i)} />
                                <label class="m-2 text-red-500 font-bold">{ &layer.name }</label>
                                </div>
                            </li>
                        });
                    }
                    layer_html
                    }
                }
                </ul>
            </div>
        }
    }
}

pub enum Msg {
    ToggleLayer(usize),
    NextDoc,
    PrevDoc,
}

pub struct App {
    corpus: teanga::Corpus,
    layers: Vec<Layer>,
    doc_no: usize,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let app = App {
            corpus: serialization::read_corpus_from_json_string(
                        "{\"_meta\":{\"text\":{\"type\":\"characters\"}},\"_order\":[\"Kjco\"],
\"Kjco\":{\"text\":\"This is a document.\"},\"abcd\":{\"text\":\"This is a second document\"}}").unwrap(),
            layers: vec![
                Layer { name: "Tokens".to_string(), selected: true },
                Layer { name: "POS".to_string(), selected: false },
            ],
            doc_no: 0,
        };
        app
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleLayer(i) => {
                self.layers[i].selected = !self.layers[i].selected;
                true
            },
            Msg::NextDoc => {
                if self.doc_no < self.corpus.documents.len() {
                    self.doc_no += 1;
                }
                true
            },
            Msg::PrevDoc => {
                if self.doc_no > 0 {
                    self.doc_no -= 1;
                }
                true
            },
        }
    }

    fn view(&self, ctx:&Context<Self>) -> Html {
        let on_layer_enable = ctx.link().callback(Msg::ToggleLayer);
        let next_doc = ctx.link().callback(|_:String| Msg::NextDoc);
        let prev_doc = ctx.link().callback(|_:String| Msg::PrevDoc);
         html! { 
            <div class="flex flex-row min-h-screen">
                <div class="bg-gray-200 basis-52">
                    <div class="p-4">
                        <h1 class="font-bold">{ "Teanga Corpus Viewer" }</h1>
                    </div>
                    <LayerSelect on_layer_enable={on_layer_enable.clone()} layers={self.layers.clone()}/>
                    <ul>{
                        self.layers.iter().map(|layer| {
                            html! {
                                <li>{ &layer.name }{ if layer.selected { "true" } else { "false" } }</li>
                            }
                        }).collect::<Html>()
                    }</ul>

                    <div class="p-4 flex flex-col">
                        <button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded m-2 inline-flex items-center">
                            <Icon icon_id={IconId::FontAwesomeSolidUpload} class={classes!("w-4", "h-4", "me-2")}/>{ "Load" }
                        </button>
                        <button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded m-2 inline-flex items-center">
                            <Icon icon_id={IconId::OcticonsBeaker24} class={classes!("w-4", "h-4", "me-2")}/>{ "Analyse" } 
                        </button>
                        <button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded m-2 inline-flex items-center">
                            <Icon icon_id={IconId::LucideSave} class={classes!("w-4", "h-4", "me-2")}/>{ "Save" }
                        </button>
                    </div>
                </div>
                <div class="bg-gray-100 grow">
                    { 
                        if self.corpus.documents.len() > 0 { 
                            html! { <DocumentView 
                                meta={self.corpus.meta.clone()}
                                layers={self.layers.clone()} document={self.corpus.documents[self.doc_no].1.clone()}
                        on_next_doc={next_doc} on_prev_doc={prev_doc}/> }
                        } else {
                            html! { <p>{ "No documents loaded" }</p> }
                        }
                    }
                </div>
            </div>
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}

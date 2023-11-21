use yew::prelude::*;
use yew_icons::{Icon, IconId};
use std::collections::HashMap;

mod teanga;
mod serialization;
mod render;

use teanga::LayerType;

#[derive(Clone, PartialEq, Properties)]
pub struct Layer {
    name: String,
    selected: bool,
    color: String
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
                    match props.document.get_annos(&props.meta)  {
                        Ok(docsecs) => {
                            docsecs.iter().map(|(name, docsec)| {
                                html! {
                                    <div class="p-4">
                                        <h3 class="font-semibold mb-4">{ name }</h3>
                                        <div class="text-sm font-medium bg-bwhite border border-gray-400 rounded-md">
                                            { render::render_annos(&docsec, props.layers.iter().map(|x| (x.name.as_str(), x.selected)).collect()) }
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                        Err(e) => html! {
                            <span>{ format!("Error: {}", e) }</span>
                        }
                    }
                }}
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
                                <input type="checkbox" checked={layer.selected} class={classes!("w-4","h-4",format!("accent-{}-900", layer.color), "rounded")}
                                onclick={move |_| on_layer_enable.emit(i)} />
                                <label class={classes!("m-2",format!("text-{}-900", layer.color), "font-bold")}>{ &layer.name }</label>
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
    ToggleModal(&'static str),
}

pub struct App {
    corpus: teanga::Corpus,
    layers: Vec<Layer>,
    doc_no: usize,
    load_modal: bool,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let mut app = App {
            corpus: serialization::read_corpus_from_json_string(
            "{\"_meta\":{\"text\":{\"type\":\"characters\"},\"tokens\":{\"type\":\"span\",\"on\":\"text\"},
\"pos\":{\"type\":\"seq\",\"on\":\"tokens\",\"data\":\"string\"}},\"_order\":[\"Kjco\"],
\"Kjco\":{\"text\":\"This is a document.\",\"tokens\":[[0,4],[5,7],[8,9],[10,19]]
,\"pos\":[\"DT\",\"VBZ\",\"DT\",\"NN\"]},
\"abcd\":{\"text\":\"This is a second document\"}}").unwrap(),
            layers: Vec::new(),
            doc_no: 0,
            load_modal: false,
        };
        app.layers = app.corpus.meta.iter().filter(|l| l.1.layer_type != LayerType::Characters).enumerate().map(|(i, (name, _))| {
            Layer {
                name: name.clone(),
                selected: false,
                color: render::COLORS[i % render::COLORS.len()].to_string(),
            }
        }).collect();
        app
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleLayer(i) => {
                self.layers[i].selected = !self.layers[i].selected;
                true
            },
            Msg::NextDoc => {
                if self.doc_no < self.corpus.documents.len() - 1 {
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
            Msg::ToggleModal(_) => {
                self.load_modal = !self.load_modal;
                true
            }
        }
    }

    fn view(&self, ctx:&Context<Self>) -> Html {
        let on_layer_enable = ctx.link().callback(Msg::ToggleLayer);
        let next_doc = ctx.link().callback(|_:String| Msg::NextDoc);
        let prev_doc = ctx.link().callback(|_:String| Msg::PrevDoc);
        let toggle_modal1 = ctx.link().callback(Msg::ToggleModal);
        let toggle_modal2 = ctx.link().callback(Msg::ToggleModal);
        let toggle_modal3 = ctx.link().callback(Msg::ToggleModal);
        let toggle_modal4 = ctx.link().callback(Msg::ToggleModal);
         html! { 
             <>
            <div class="flex flex-row min-h-screen">
                <div class="bg-gray-200 basis-52">
                    <div class="p-4">
                        <h1 class="font-bold">{ "Teanga Corpus Viewer" }</h1>
                    </div>
                    <LayerSelect on_layer_enable={on_layer_enable.clone()} layers={self.layers.clone()}/>

                    <div class="p-4 flex flex-col">
                        <button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded m-2 inline-flex items-center"
                        onclick={move |_| toggle_modal1.emit("load")}>
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
          <div class={{
              if self.load_modal {
                  classes!("fixed", "w-full", "h-full", "top-0", "left-0", "flex", "items-center", "justify-center")
              } else {
                  classes!("opacity-0", "pointer-events-none", "fixed", "w-full", "h-full", "top-0", "left-0", "flex", "items-center", "justify-center")
              }
          }}>
                <div class="modal-overlay absolute w-full h-full bg-gray-900 opacity-50"></div>

                    <div class="modal-container bg-white w-11/12 md:max-w-md mx-auto rounded shadow-lg z-50 overflow-y-auto">

                    <div class="modal-close absolute top-0 right-0 cursor-pointer flex flex-col items-center mt-4 mr-4 text-white text-sm z-50" onclick={move |_| toggle_modal2.emit("load")}>
                    <svg class="fill-current text-white" xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 18 18">
                    <path d="M14.53 4.53l-1.06-1.06L9 7.94 4.53 3.47 3.47 4.53 7.94 9l-4.47 4.47 1.06 1.06L9 10.06l4.47 4.47 1.06-1.06L10.06 9z"></path>
                    </svg>
                    </div>

                    <div class="modal-content py-4 text-left px-6">
                    <div class="flex justify-between items-center pb-3">
                    <p class="text-2xl font-bold">{ "Load a corpus" }</p>
                    <div class="modal-close cursor-pointer z-50" onclick={move |_| toggle_modal3.emit("load")}>
                    <svg class="fill-current text-black" xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 18 18">
                    <path d="M14.53 4.53l-1.06-1.06L9 7.94 4.53 3.47 3.47 4.53 7.94 9l-4.47 4.47 1.06 1.06L9 10.06l4.47 4.47 1.06-1.06L10.06 9z"></path>
                    </svg>
                    </div>
                    </div>

                    <p>{ "Modal content can go here" }</p>

                    <div class="flex justify-end pt-2">
                    <button class="modal-close px-4 bg-indigo-500 p-3 rounded-lg text-white hover:bg-indigo-400" onclick={move |_| toggle_modal4.emit("load")}>{ "Load" }</button>
                    </div>

                    </div>
                </div>
            </div>
        </>
         }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}

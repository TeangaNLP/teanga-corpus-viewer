/// The Teanga data model as implemented by this model
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fmt::{self, Display, Formatter};

#[derive(Debug,Clone)]
/// A corpus object
pub struct Corpus {
    pub meta: HashMap<String, LayerDesc>,
    pub order: Vec<String>,
    pub documents: Vec<(String, Document)>,
}

impl Corpus {
    pub fn new() -> Self {
        Corpus {
            meta: HashMap::new(),
            order: Vec::new(),
            documents: Vec::new(),
        }
    }
}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
/// A layer description
pub struct LayerDesc {
    #[serde(rename = "type")]
    pub layer_type: LayerType,
    #[serde(default = "String::new")]
    #[serde(skip_serializing_if = "String::is_empty")]
    pub on: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<DataType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Vec<String>>,
}

pub struct DocSecs<'a,'b> {
    pub content : &'a str,
    pub annos : Vec<Vec<Anno<'a,'b>>>
}

pub struct Anno<'a,'b> {
    pub layer_name : &'b str,
    pub data : Option<&'a Data>,
    pub left_complete : bool,
    pub right_complete : bool,
    pub left_idx : usize,
    pub right_idx : usize
}

impl<'a,'b> Anno<'a,'b> {
    pub fn new(layer_name : &'b str,  data : Option<&'a Data>, left_idx : usize, right_idx : usize) -> Self {
        Anno {
            layer_name,
            data,
            left_complete: true,
            right_complete: true,
            left_idx,
            right_idx
        }
    }
}

#[derive(Debug,Clone, PartialEq)]
/// A document object
pub struct Document {
    pub content: HashMap<String, Layer>
}

impl Document {
    pub fn new() -> Self {
        Document {
            content: HashMap::new()
        }
    }

    pub fn get_text_layers(&self) -> HashMap<String, &String> {
        let mut text_layers = HashMap::new();
        for (layer_name, layer) in self.content.iter() {
            match layer {
                Layer::Characters(s) => {
                    text_layers.insert(layer_name.clone(), s);
                },
                _ => ()
            }
        }
        text_layers
    }

    pub fn get_annos<'a,'b>(&'a self, meta : &'b HashMap<String, LayerDesc>) -> Result<HashMap<String, DocSecs<'a,'b>>, String> where 'a: 'b {
        let mut annos = HashMap::new();
        let mut base_annos = HashMap::new();
        for (layer_name, layer) in self.content.iter() {
            match layer {
                Layer::Characters(s) => {
                    annos.insert(layer_name.clone(), DocSecs {
                        content : &s,
                        annos : Vec::new()
                    });
                },
                _ => {
                    let (base, on) = self.base_annos(layer_name, meta)?;
                    base_annos.entry(on).or_insert_with(Vec::new).extend(base);
                }
            }
        }
        for (base_layer_name, doc_secs) in annos.iter_mut() {
            let base_annos = base_annos.entry(base_layer_name).or_insert_with(Vec::new);
            let mut last_i = 0;
            for (i,j) in calc_divisions(&base_annos) {
                for anno in base_annos.iter() {
                    if anno.left_idx >= i && anno.right_idx <= j {
                        let anno2 = if anno.left_idx == i && anno.right_idx == j {
                            Anno::new(anno.layer_name, anno.data, i, j)
                        } else if anno.left_idx == i {
                            let mut anno = Anno::new(anno.layer_name, anno.data, i, j);
                            anno.left_complete = false;
                            anno
                        } else if anno.right_idx == j {
                            let mut anno = Anno::new(anno.layer_name, anno.data, i, j);
                            anno.right_complete = false;
                            anno
                        } else {
                            let mut anno = Anno::new(anno.layer_name, anno.data, i, j);
                            anno.left_complete = false;
                            anno.right_complete = false;
                            anno
                        };
                        if i == last_i && doc_secs.annos.len() > 0 {
                            let n = doc_secs.annos.len() - 1;
                            doc_secs.annos[n].push(anno2);
                        } else {
                            doc_secs.annos.push(vec![anno2]);
                        }
                     }
                }
                last_i = i;
           }
        }

        Ok(annos)
    }

    fn base_annos<'a,'b>(&'a self, name : &'b str, meta : &'b HashMap<String, LayerDesc>) -> Result<(Vec<Anno<'a,'b>>, &'b str),String> {
        let layer = self.content.get(name).ok_or_else(|| format!("No layer {}", name))?;
        let this_meta = meta.get(name).ok_or_else(|| format!("No meta data for layer {}", name))?;
        match layer {
            Layer::Characters(_) => Err("Base index cannot be called on a character layer".to_string()),
            Layer::Seq(_) => {
                match self.content.get(&this_meta.on).ok_or_else(|| format!("No data for layer {}", name))? {
                    Layer::Characters(s) =>
                        Ok(((0..s.len()).map(|i|
                                Anno::new(name, None, i,i+1)).collect(), &this_meta.on)),
                    _ => 
                        self.base_annos(&this_meta.on, meta)
                }
            },
            Layer::Div(data) => {
                match self.content.get(&this_meta.on).ok_or_else(|| format!("No data for layer {}", name))? {
                    Layer::Characters(s) => {
                        let mut base = Vec::new();
                        let mut start : Option<usize> = None;
                        let mut last_d = None;
                        for (i,d) in data.iter() {
                            match start {
                                Some(start) => base.push(Anno::new(name, last_d, start, *i)),
                                None => {}
                            }
                            start = Some(*i);
                            last_d = Some(d);
                        }
                        match start {
                            Some(start) => base.push(Anno::new(name, last_d, start, s.len())),
                            None => {}
                        }
                        Ok((base, &this_meta.on))
                    },
                    _ => { 
                        let (indexes, on) = self.base_annos(&this_meta.on, meta)?;
                        let mut base = Vec::new();
                        let mut start = None;
                        let mut last_d = None;
                        for (i,d) in data.iter() {
                            match start {
                                Some(start) => base.push(Anno::new(name, last_d, start, indexes[*i].right_idx)),
                                None => {}
                            }
                            start = Some(indexes[*i].left_idx);
                            last_d = Some(d);
                        }
                        match start {
                            Some(start) => base.push(Anno::new(name, 
                                    last_d, start, indexes[indexes.len()-1].right_idx)),
                            None => {}
                        }
                        Ok((base,on))
                    }
                }
            },
            Layer::DivNoData(data) => {
                match self.content.get(&this_meta.on).ok_or_else(|| format!("No data for layer {}", name))? {
                    Layer::Characters(s) => {
                        let mut base = Vec::new();
                        let mut start : Option<usize> = None;
                        for i in data.iter() {
                            match start {
                                Some(start) => base.push(Anno::new(name, None, start, *i)),
                                None => {}
                            }
                            start = Some(*i);
                        }
                        match start {
                            Some(start) => base.push(Anno::new(name, None, start, s.len())),
                            None => {}
                        }
                        Ok((base, &this_meta.on))
                    },
                    _ => { 
                        let (indexes, on) = self.base_annos(&this_meta.on, meta)?;
                        let mut base = Vec::new();
                        let mut start = None;
                        for i in data.iter() {
                            match start {
                                Some(start) => base.push(Anno::new(name, None, start, indexes[*i].right_idx)),
                                None => {}
                            }
                            start = Some(indexes[*i].left_idx);
                        }
                        match start {
                            Some(start) => base.push(Anno::new(name, None, start, indexes[indexes.len()-1].right_idx)),
                            None => {}
                        }
                        Ok((base,on))
                    }
                }
             },
             Layer::Element(data) => {
                match self.content.get(&this_meta.on).ok_or_else(|| format!("No data for layer {}", name))? {
                    Layer::Characters(_) => {
                        let mut base = Vec::new();
                        for (i,d) in data.iter() {
                            base.push(Anno::new(name, Some(d), *i, i+1));
                        }
                        Ok((base, &this_meta.on))
                    },
                    _ => {
                        let (indexes, on) = self.base_annos(&this_meta.on, meta)?;
                        let mut base = Vec::new();
                        for (i,d) in data.iter() {
                            base.push(Anno::new(name, Some(d), indexes[*i].left_idx, indexes[*i].right_idx));
                        }
                        Ok((base,on))
                    }
                }
             },
             Layer::ElementNoData(data) => {
                match self.content.get(&this_meta.on).ok_or_else(|| format!("No data for layer {}", name))? {
                    Layer::Characters(_) => {
                        let mut base = Vec::new();
                        for i in data.iter() {
                            base.push(Anno::new(name, None, *i, i+1));
                        }
                        Ok((base, &this_meta.on))
                    },
                    _ => {
                        let (indexes, on) = self.base_annos(&this_meta.on, meta)?;
                        let mut base = Vec::new();
                        for i in data.iter() {
                            base.push(Anno::new(name, None, indexes[*i].left_idx, indexes[*i].right_idx));
                        }
                        Ok((base,on))
                    }
                }
             },
             Layer::Span(data) => {
                match self.content.get(&this_meta.on).ok_or_else(|| format!("No data for layer {}", name))? {
                    Layer::Characters(_) => {
                        let mut base = Vec::new();
                        for (i,j,d) in data.iter() {
                            base.push(Anno::new(name, Some(d), *i, *j));
                        }
                        Ok((base, &this_meta.on))
                    },
                    _ => {
                        let (indexes, on) = self.base_annos(&this_meta.on, meta)?;
                        let mut base = Vec::new();
                        for (i,j,d) in data.iter() {
                            base.push(Anno::new(name, Some(d), indexes[*i].left_idx, indexes[*j-1].right_idx));
                        }
                        Ok((base,on))
                    }
                }
             },
             Layer::SpanNoData(data) => {
                match self.content.get(&this_meta.on).ok_or_else(|| format!("No data for layer {}", name))? {
                    Layer::Characters(_) => {
                        let mut base = Vec::new();
                        for (i,j) in data.iter() {
                            base.push(Anno::new(name, None, *i, *j));
                        }
                        Ok((base, &this_meta.on))
                    },
                    _ => {
                        let (indexes, on) = self.base_annos(&this_meta.on, meta)?;
                        let mut base = Vec::new();
                        for (i,j) in data.iter() {
                            base.push(Anno::new(name, None, indexes[*i].left_idx, indexes[*j-1].right_idx));
                        }
                        Ok((base,on))
                    }
                }
             }
        }
    }
}

fn calc_divisions<'a,'b>(annos : &Vec<Anno<'a,'b>>) -> Vec<(usize, usize)> {
    let mut divisions = Vec::new();
    for a in annos.iter() {
        let mut overlaps = Vec::new();
        for (i,j) in divisions.iter() {
            if a.left_idx < *j && a.right_idx > *i {
                overlaps.push((*i,*j));
            }
        }
        for (i,j) in overlaps.iter() {
            divisions.retain(|(x,y)| x != i && y != j);
        }
        let mut start = a.left_idx;
        let mut end = 0;
        for (i,j) in overlaps.iter() {
            if start < *i {
                divisions.push((start, *i));
            }
            divisions.push((*i, *j));
            start = *i;
            end = *j;
        }
        if end < a.right_idx {
            divisions.push((end, a.right_idx));
        }
    }
    divisions.sort();
    divisions
}

#[derive(Debug,Clone, PartialEq)]
pub enum Layer {
    Characters(String),
    Seq(Vec<Data>),
    Div(Vec<(usize,Data)>),
    DivNoData(Vec<usize>),
    Element(Vec<(usize,Data)>),
    ElementNoData(Vec<usize>),
    Span(Vec<(usize,usize,Data)>),
    SpanNoData(Vec<(usize,usize)>),
}


#[derive(Debug,Clone,PartialEq)]
pub enum Data {
    String(String),
    Link(usize),
    TypedLink(usize, String),
}

impl Data {
    pub fn from_str(s : String) -> Data {
        Data::String(s)
    }

    pub fn from_usize(u : usize) -> Data {
        Data::Link(u)
    }

    pub fn from_link(u : usize, s : String) -> Data {
        Data::TypedLink(u, s)
    }

    pub fn into_str(self) -> Option<String> {
        match self {
            Data::String(s) => Some(s),
            _ => None
        }
    }

    pub fn into_usize(self) -> Option<usize> {
        match self {
            Data::Link(u) => Some(u),
            _ => None
        }
    }

    pub fn into_link(self) -> Option<(usize, String)> {
        match self {
            Data::TypedLink(u, s) => Some((u, s)),
            _ => None
        }
    }
}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum LayerType {
    #[serde(rename = "characters")]
    Characters,
    #[serde(rename = "seq")]
    Seq,
    #[serde(rename = "div")]
    Div,
    #[serde(rename = "element")]
    Element,
    #[serde(rename = "span")]
    Span
}

impl Display for LayerType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LayerType::Characters => write!(f, "characters"),
            LayerType::Seq => write!(f, "seq"),
            LayerType::Div => write!(f, "div"),
            LayerType::Element => write!(f, "element"),
            LayerType::Span => write!(f, "span")
        }
    }
}



#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum DataType {
    String,
    Enum(Vec<String>),
    Link,
    TypedLink(Vec<String>)
}

impl Display for DataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DataType::String => write!(f, "string"),
            DataType::Enum(_) => write!(f, "enum"),
            DataType::Link => write!(f, "link"),
            DataType::TypedLink(_) => write!(f, "typedlink")
        }
    }
}


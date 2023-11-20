/// The Teanga data model as implemented by this model
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fmt::{self, Display, Formatter};
use std::cmp::Ordering;

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

#[derive(Debug)]
pub struct DocSecs<'a,'b> {
    pub content : &'a str,
    pub annos : Vec<Anno<'a,'b>>
}

#[derive(Debug, PartialEq, Clone)]
pub struct Anno<'a,'b> {
    pub layer_name : &'b str,
    pub data : Option<&'a Data>,
    pub left_complete : bool,
    pub right_complete : bool,
    pub start : usize,
    pub end : usize,
    pub children : Vec<Anno<'a,'b>>,
}

impl<'a,'b> Anno<'a,'b> {
    pub fn new(layer_name : &'b str,  data : Option<&'a Data>, start : usize, end : usize) -> Self {
        Anno {
            layer_name,
            data,
            left_complete: true,
            right_complete: true,
            start,
            end,
            children: Vec::new()
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
        let layer_tree = LayerTree::from_meta(meta);
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
            let mut annos2 = Vec::new();
            let divisions = calc_divisions(&base_annos);
            'anno_loop: for anno in base_annos.iter() {
                for (i,j) in divisions.iter() {
                    let i = *i;
                    let j = *j;
                    if anno.start >= i && anno.end <= j {
                        let anno2 = if anno.start == i && anno.end == j {
                            Anno::new(anno.layer_name, anno.data, i, j)
                        } else if anno.start == i {
                            let mut anno = Anno::new(anno.layer_name, anno.data, i, j);
                            anno.left_complete = false;
                            anno
                        } else if anno.end == j {
                            let mut anno = Anno::new(anno.layer_name, anno.data, i, j);
                            anno.right_complete = false;
                            anno
                        } else {
                            let mut anno = Anno::new(anno.layer_name, anno.data, i, j);
                            anno.left_complete = false;
                            anno.right_complete = false;
                            anno
                        };
                        annos2.push(anno2);
                        continue 'anno_loop;
                     }
                }
           }
           
            annos2.sort_by(|a,b| {
                if a.start < b.start {
                    std::cmp::Ordering::Less
                } else if a.start > b.start {
                    std::cmp::Ordering::Greater
                } else if a.end < b.end {
                    std::cmp::Ordering::Greater
                } else if a.end > b.end {
                    std::cmp::Ordering::Less
                } else {
                    layer_tree.cmp_str(a.layer_name, b.layer_name)
                }
            });
           doc_secs.annos = merge_annos_recursively(annos2);
        }

        Ok(annos)
    }

    fn base_annos<'a,'b>(&'a self, name : &'b str, meta : &'b HashMap<String, LayerDesc>) -> Result<(Vec<Anno<'a,'b>>, &'b str),String> {
        let layer = self.content.get(name).ok_or_else(|| format!("No layer {}", name))?;
        let this_meta = meta.get(name).ok_or_else(|| format!("No meta data for layer {}", name))?;
        match layer {
            Layer::Characters(_) => Err("Base index cannot be called on a character layer".to_string()),
            Layer::Seq(data) => {
                match self.content.get(&this_meta.on).ok_or_else(|| format!("No data for layer {}", name))? {
                    Layer::Characters(s) =>
                        Ok(((0..s.len()).map(|i|
                                Anno::new(name, None, i,i+1)).collect(), &this_meta.on)),
                    _ => {
                        let mut base = Vec::new();
                        let (annos, on) = self.base_annos(&this_meta.on, meta)?;
                        for (a, d) in annos.iter().zip(data.iter()) {
                            base.push(Anno::new(name, Some(d), a.start, a.end));
                        }
                        Ok((base, on))
                    }
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
                                Some(start) => base.push(Anno::new(name, last_d, start, indexes[*i].end)),
                                None => {}
                            }
                            start = Some(indexes[*i].start);
                            last_d = Some(d);
                        }
                        match start {
                            Some(start) => base.push(Anno::new(name, 
                                    last_d, start, indexes[indexes.len()-1].end)),
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
                                Some(start) => base.push(Anno::new(name, None, start, indexes[*i].end)),
                                None => {}
                            }
                            start = Some(indexes[*i].start);
                        }
                        match start {
                            Some(start) => base.push(Anno::new(name, None, start, indexes[indexes.len()-1].end)),
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
                            base.push(Anno::new(name, Some(d), indexes[*i].start, indexes[*i].end));
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
                            base.push(Anno::new(name, None, indexes[*i].start, indexes[*i].end));
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
                            base.push(Anno::new(name, Some(d), indexes[*i].start, indexes[*j-1].end));
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
                            base.push(Anno::new(name, None, indexes[*i].start, indexes[*j-1].end));
                        }
                        Ok((base,on))
                    }
                }
             }
        }
    }
}

#[derive(Debug,Clone,PartialEq)]
struct LayerTree {
    data : HashMap<String, LayerTree>
}

impl LayerTree {
    pub fn from_meta(meta : &HashMap<String, LayerDesc>) -> LayerTree {
       let mut all_data = HashMap::new();
       all_data.insert("".to_string(), Vec::new());
       for (name, desc) in meta.iter() {
            all_data.entry(desc.on.clone()).or_insert_with(Vec::new).push(name.clone());
       }
       fn build_data(name : &str, all_data : &HashMap<String, Vec<String>> ) -> LayerTree {
           let mut data = HashMap::new();
           if let Some(names) = all_data.get(name) {
               for name in names {
                   data.insert(name.clone(), build_data(name, all_data));
               }
           }
           LayerTree { data }
       }
       build_data("", &all_data)
    }
    
    pub fn contains(&self, a : &str) -> bool {
        self.data.contains_key(a) || self.data.values().any(|v| v.contains(a))
    }

    pub fn cmp_str(&self, a : &str, b: &str) -> Ordering {
        if self.data.contains_key(a) && self.data.values().any(|v| v.contains(b)) {
            Ordering::Less
        } else if self.data.contains_key(b) && self.data.values().any(|v| v.contains(a)) {
            Ordering::Greater
        } else {
            for (_,v) in self.data.iter() {
                if v.contains(a) && v.contains(b) {
                    return v.cmp_str(a,b);
                }
            }
            a.cmp(b)
        }
    }
}

fn merge_annos_recursively<'a,'b>(annos : Vec<Anno<'a, 'b>>) -> Vec<Anno<'a,'b>> {
    let mut new_annos = Vec::new();
    let mut span_i = 0;
    let mut span_j = 0;
    let mut batch = Vec::new();
    let mut batch_anno : Option<Anno<'a,'b>> = None;
    for anno in annos {
        // Anno is in overlap with the current batch
        if span_j != 0 && anno.start <= span_j && anno.end >= span_i {
            batch.push(anno);
        } else {
            if let Some(mut batch_anno) = batch_anno {
                batch_anno.children = merge_annos_recursively(batch);
                new_annos.push(batch_anno);
            }
            batch = Vec::new();
            span_i = anno.start;
            span_j = anno.end;
            batch_anno = Some(anno);
        }
    }
    if let Some(mut batch_anno) = batch_anno {
        batch_anno.children = merge_annos_recursively(batch);
        new_annos.push(batch_anno);
    }
    new_annos
}

fn calc_divisions<'a,'b>(annos : &Vec<Anno<'a,'b>>) -> Vec<(usize, usize)> {
    let mut divisions = annos.iter().map(|a| (a.start, a.end)).collect::<Vec<(usize,usize)>>();
    'outer: loop {
        // We are looking for overlaps
        //    i.0     i.1
        //    -----------
        //    |         |
        //    -----------
        //        -----------
        //        |         |
        //        -----------
        //        j.0     j.1
        //
        // and map them to three blocks
        //
        //   i.0  j.0  i.1 j.1
        //   -----------------
        //   |   |   |   |   |
        //   -----------------
        for i in 0..divisions.len() {
            for j in i+1..divisions.len() {
                if divisions[j].0 > divisions[i].0 && divisions[j].0 < divisions[i].1 
                    && divisions[j].1 > divisions[i].1 {
                        let (i0, i1) = divisions.remove(i);
                        let (j0, j1) = divisions.remove(j-1);
                        divisions.push((i0, j0));
                        divisions.push((j0, j1));
                        divisions.push((i1, j1));
                        continue 'outer;
                }
            }
        }
        break;
    }
    // Sort by start and the by end in reverse order
    divisions.sort_by(|a,b| {
        if a.0 < b.0 {
            std::cmp::Ordering::Less
        } else if a.0 > b.0 {
            std::cmp::Ordering::Greater
        } else if a.1 < b.1 {
            std::cmp::Ordering::Greater
        } else if a.1 > b.1 {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
    });
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



#[derive(Debug,Clone,PartialEq)]
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_base_annos() {
        let corpus = crate::serialization::read_corpus_from_json_string(
            "{\"_meta\":{\"text\":{\"type\":\"characters\"},\"tokens\":{\"type\":\"span\",\"on\":\"text\"}},\"_order\":[\"Kjco\"],
\"Kjco\":{\"text\":\"This is a document.\",\"tokens\":[[0,4],[5,7],[8,9],[10,19]]},
\"abcd\":{\"text\":\"This is a second document\"}}").unwrap();
        let doc = &corpus.documents[0].1;
        let meta = &corpus.meta;
        let (base, on) = doc.base_annos("tokens", meta).unwrap();
        assert_eq!(on, "text");
        assert_eq!(base.len(), 4);
        assert_eq!(base[0].start, 0);
        assert_eq!(base[0].end, 4);
        assert_eq!(base[1].start, 5);
        assert_eq!(base[1].end, 7);
    }

    #[test]
    fn test_calc_divisions() {
        let corpus = crate::serialization::read_corpus_from_json_string(
            "{\"_meta\":{\"text\":{\"type\":\"characters\"},\"tokens\":{\"type\":\"span\",\"on\":\"text\"}},\"_order\":[\"Kjco\"],
\"Kjco\":{\"text\":\"This is a document.\",\"tokens\":[[0,4],[5,7],[8,9],[10,19]]},
\"abcd\":{\"text\":\"This is a second document\"}}").unwrap();
        let doc = &corpus.documents[0].1;
        let meta = &corpus.meta;
        let (base, on) = doc.base_annos("tokens", meta).unwrap();
        let divisions = calc_divisions(&base);
        assert_eq!(divisions.len(), 4);
        assert_eq!(divisions[0].0, 0);
        assert_eq!(divisions[0].1, 4);
        assert_eq!(divisions[1].0, 5);
        assert_eq!(divisions[1].1, 7);
        assert_eq!(divisions[2].0, 8);
        assert_eq!(divisions[2].1, 9);
        assert_eq!(divisions[3].0, 10);
        assert_eq!(divisions[3].1, 19);
    }

    #[test]
    fn test_merge_annos_recursively() {

        let annos = vec![
            Anno { layer_name: "tokens", data: None, left_complete: true, 
                right_complete: true, start: 0, end: 4, children: Vec::new() },
            Anno { layer_name: "tokens", data: None, left_complete: true, 
                right_complete: true, start: 5, end: 7, children: Vec::new() },
            Anno { layer_name: "tokens", data: None, left_complete: true, 
                right_complete: true, start: 9, end: 10, children: Vec::new() },
            Anno { layer_name: "tokens", data: None, left_complete: true, 
                right_complete: true, start: 11, end: 19, children: Vec::new() }];
        let results = merge_annos_recursively(annos.clone());
        assert_eq!(annos, results);
    }

    #[test]
    fn test_layer_tree() {
        let mut meta = HashMap::new();
        meta.insert("text".to_string(), LayerDesc {
            layer_type: LayerType::Characters,
            on: "".to_string(),
            data: None,
            values: None,
            target: None,
            default: None
        });
        meta.insert("tokens".to_string(), LayerDesc {
            layer_type: LayerType::Span,
            on: "text".to_string(),
            data: None,
            values: None,
            target: None,
            default: None
        });
        meta.insert("pos".to_string(), LayerDesc {
            layer_type: LayerType::Element,
            on: "tokens".to_string(),
            data: None,
            values: None,
            target: None,
            default: None
        });
        let layer_tree = LayerTree::from_meta(&meta);
        eprintln!("{:?}", layer_tree);
        assert!(layer_tree.contains("text"));
        assert_eq!(layer_tree.cmp_str("tokens", "pos"), Ordering::Less);
    }
}

use thiserror::Error;
use serde::{Serialize, Deserialize, Deserializer};
use crate::teanga::{LayerDesc, LayerType, DataType, Layer, Data, Corpus, Document};
use serde::ser::{SerializeMap, Serializer};
use serde::de::Visitor;
use std::collections::HashMap;

struct TeangaVisitor();

impl<'de> Visitor<'de> for TeangaVisitor {
    type Value = Corpus;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string representing a corpus")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where A: serde::de::MapAccess<'de>
    {
        let mut corpus = Corpus::new();
        while let Some(ref key) = map.next_key::<String>()? {
            if key == "_meta" {
                let data = map.next_value::<HashMap<String, LayerDesc>>()?;
                corpus.meta = data;
            } else if key == "_order" {
                corpus.order = map.next_value::<Vec<String>>()?;
            } else {
                let doc = map.next_value::<HashMap<String, PyLayer>>()?;
                let mut mapped_doc = HashMap::new();
                for (id, layer) in &doc {
                    let meta : &LayerDesc = corpus.meta.get(id).ok_or_else
                        (|| serde::de::Error::custom(format!("No meta for layer {}", id)))?;
                    mapped_doc.insert(id.clone(), Layer::from_py(layer.clone(), meta)
                        .map_err(serde::de::Error::custom)?);
                }
                corpus.documents.push((key.clone(), Document { content: mapped_doc }));
            }
        }
        Ok(corpus)
    }
}

pub fn read_corpus_from_json_string(s: &str) -> Result<Corpus, serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_str(s);
    deserializer.deserialize_any(TeangaVisitor())
}

pub fn write_corpus_to_json_string(corpus: &Corpus) -> Result<String, TeangaError> {
    let mut ser = serde_json::Serializer::new(Vec::new());
    corpus.serialize(&mut ser)?;
    Ok(String::from_utf8(ser.into_inner())?)
}



impl Serialize for Corpus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("_meta", &self.meta)?;
        map.serialize_entry("_order", &self.order)?;
        for (id, doc) in &self.documents {
            let mut mapped_doc = HashMap::new();
            for (id, layer) in &doc.content {
                let meta : &LayerDesc = self.meta.get(id).ok_or_else
                    (|| serde::ser::Error::custom(format!("No meta for layer {}", id)))?;
                mapped_doc.insert(id.clone(), layer.into_py(meta)
                    .map_err(serde::ser::Error::custom)?);
            }
            map.serialize_entry(id, &mapped_doc)?;
        }
        map.end()
    }
}

#[derive(Debug,Clone,PartialEq,Serialize, Deserialize)]
#[serde(untagged)]
enum PyLayer {
    CharacterLayer(String),
    L1(Vec<usize>),
    L2(Vec<(usize,usize)>),
    L3(Vec<(usize,usize,usize)>),
    LS(Vec<String>),
    L1S(Vec<(usize,String)>),
    L2S(Vec<(usize,usize,String)>),
    L3S(Vec<(usize,usize,usize,String)>),
}


impl Layer {
    fn into_py(&self, meta : &LayerDesc) -> TeangaResult<PyLayer> {
        match self {
            Layer::Characters(val) => Ok(PyLayer::CharacterLayer(val.clone())),
            Layer::Seq(val) => {
                match meta.data {
                    None => Err(TeangaError::ModelError(
                        format!("Layer contains data but not data type"))),
                    Some(DataType::String) => {
                        let mut result = Vec::new();
                        for id in val {
                            result.push(id.clone().into_str().ok_or_else(|| TeangaError::ModelError(
                                format!("String layer contains non-string data")))?);
                        }
                        Ok(PyLayer::LS(result))
                    },
                    Some(DataType::Enum(_)) => {
                        let mut result = Vec::new();
                        for id in val {
                            result.push(id.clone().into_str().ok_or_else(|| TeangaError::ModelError(
                                format!("String layer contains non-string data")))?);
                        }
                        Ok(PyLayer::LS(result))
                    },
                    Some(DataType::Link) => {
                        let mut result = Vec::new();
                        for d in val {
                            result.push(d.clone().into_usize().ok_or_else(|| TeangaError::ModelError(
                                format!("Link layer contains non-link data")))?);
                        }
                        Ok(PyLayer::L1(result))
                    },
                    Some(DataType::TypedLink(_)) => {
                        let mut result = Vec::new();
                        for id in val {
                            result.push(id.clone().into_link().ok_or_else(|| TeangaError::ModelError(
                                format!("Typed link layer contains non-link data")))?);
                        }
                        Ok(PyLayer::L1S(result))
                    }
                }
            },
            Layer::Div(val) => {
                match meta.data {
                    None => Err(TeangaError::ModelError(
                        format!("Layer contains data but no data type"))),
                    Some(DataType::String) => {
                        let mut result = Vec::new();
                        for (start, data) in val {
                            result.push((*start, 
                                    data.clone().into_str().ok_or_else(|| TeangaError::ModelError(
                                        format!("String layer contains non-string data")))?));
                        }
                        Ok(PyLayer::L1S(result))
                    },
                    Some(DataType::Enum(_)) => {
                        let mut result = Vec::new();
                        for (start, data) in val {
                            result.push((*start, data.clone().into_str().ok_or_else(|| TeangaError::ModelError(
                                format!("String layer contains non-string data")))?));
                        }
                        Ok(PyLayer::L1S(result))
                    },
                    Some(DataType::Link) => {
                        let mut result = Vec::new();
                        for (start, data) in val {
                            result.push((*start, 
                                    data.clone().into_usize().ok_or_else(|| TeangaError::ModelError(
                                        format!("Link layer contains non-link data")))?));
                        }
                        Ok(PyLayer::L2(result))
                    },
                    Some(DataType::TypedLink(_)) => {
                        let mut result = Vec::new();
                        for (start, data) in val {
                            let tl = data.clone().into_link().ok_or_else(|| TeangaError::ModelError(
                                format!("Typed link layer contains non-link data")))?;
                            result.push((*start, tl.0, tl.1));
                        }
                        Ok(PyLayer::L2S(result))
                    }
                }
            },
            Layer::Element(val) => {
                match meta.data {
                    None => Err(TeangaError::ModelError(
                        format!("Layer contains data but no data type"))),
                    Some(DataType::String) => {
                        let mut result = Vec::new();
                        for (start, data) in val {
                            result.push((*start, data.clone().into_str().ok_or_else(|| TeangaError::ModelError(
                                format!("String layer contains non-string data")))?));
                        }
                        Ok(PyLayer::L1S(result))
                    },
                    Some(DataType::Enum(_)) => {
                        let mut result = Vec::new();
                        for (start, data) in val {
                            result.push((*start, data.clone().into_str().ok_or_else(|| TeangaError::ModelError(
                                format!("String layer contains non-string data")))?));
                        }
                        Ok(PyLayer::L1S(result))
                    },
                    Some(DataType::Link) => {
                        let mut result = Vec::new();
                        for (start, data) in val {
                            result.push((*start, data.clone().into_usize().ok_or_else(|| TeangaError::ModelError(
                                format!("Link layer contains non-link data")))?));
                        }
                        Ok(PyLayer::L2(result))
                    },
                    Some(DataType::TypedLink(_)) => {
                        let mut result = Vec::new();
                        for (start, data) in val {
                            let tl = data.clone().into_link().ok_or_else(|| TeangaError::ModelError(
                                format!("Typed link layer contains non-link data")))?;
                            result.push((*start, tl.0, tl.1));
                        }
                        Ok(PyLayer::L2S(result))
                    }
                }
            },
            Layer::Span(val) => {
                match meta.data {
                    None => Err(TeangaError::ModelError(
                        format!("Layer contains data but no data type"))),
                    Some(DataType::String) => {
                        let mut result = Vec::new();
                        for (start, end, data) in val {
                            result.push((*start, *end, 
                                    data.clone().into_str().ok_or_else(|| TeangaError::ModelError(
                                        format!("String layer contains non-string data")))?));
                        }
                        Ok(PyLayer::L2S(result))
                    },
                    Some(DataType::Enum(_)) => {
                        let mut result = Vec::new();
                        for (start, end, data) in val {
                            result.push((*start, *end, 
                                    data.clone().into_str().ok_or_else(|| TeangaError::ModelError(
                                        format!("String layer contains non-string data")))?));
                        }
                        Ok(PyLayer::L2S(result))
                    },
                    Some(DataType::Link) => {
                        let mut result = Vec::new();
                        for (start, end, data) in val {
                            result.push((*start, *end, 
                                    data.clone().into_usize().ok_or_else(|| TeangaError::ModelError(
                                        format!("Link layer contains non-link data")))?));
                        }
                        Ok(PyLayer::L3(result))
                    },
                    Some(DataType::TypedLink(_)) => {
                        let mut result = Vec::new();
                        for (start, end, data) in val {
                            let tl = data.clone().into_link().ok_or_else(|| TeangaError::ModelError(
                                format!("Typed link layer contains non-link data")))?;
                            result.push((*start, *end, tl.0, tl.1));
                        }
                        Ok(PyLayer::L3S(result))
                    }
                }
            },
            Layer::DivNoData(val) => {
                let mut result = Vec::new();
                for start in val {
                    result.push(*start);
                }
                Ok(PyLayer::L1(result))
            },
            Layer::ElementNoData(val) => {
                let mut result = Vec::new();
                for start in val {
                    result.push(*start);
                }
                Ok(PyLayer::L1(result))
            },
            Layer::SpanNoData(val) => {
                let mut result = Vec::new();
                for (start, end) in val {
                    result.push((*start, *end));
                }
                Ok(PyLayer::L2(result))
            },
        }
    }

    fn from_py(obj : PyLayer, meta : &LayerDesc) -> TeangaResult<Layer> {
        match obj {
            PyLayer::CharacterLayer(val) => Ok(Layer::Characters(val)),
            PyLayer::L1(val) => {
                match meta.data {
                    Some(_) => {
                        Ok(Layer::Seq(val.into_iter().map(|x| Data::from_usize(x)).collect()))
                    },
                    None => {
                        match meta.layer_type {
                            LayerType::Div => Ok(Layer::DivNoData(
                                    val.into_iter().map(|x| x as usize).collect())),
                            LayerType::Element => Ok(Layer::ElementNoData(
                                    val.into_iter().map(|x| x as usize).collect())),
                            _ => Err(TeangaError::ModelError(
                                format!("Cannot convert data layer to {}", meta.layer_type)))
                        }
                    }
                }
            },
            PyLayer::L2(val) => {   
                match meta.data {
                    Some(_) => {
                        match meta.layer_type {
                            LayerType::Div => Ok(Layer::Div(
                                    val.into_iter().map(|(start, end)| 
                                        (start, Data::from_usize(end))).collect())),
                            LayerType::Element => Ok(Layer::Element(
                                    val.into_iter().map(|(start, end)| 
                                        (start, Data::from_usize(end))).collect())),
                            _ => Err(TeangaError::ModelError(
                                format!("Cannot convert data layer to {}", meta.layer_type)))
                        }
                    },
                    None => {
                        Ok(Layer::SpanNoData(
                                val.into_iter().map(|(start, end)| 
                                    (start as usize, end as usize)).collect()))
                    }
                }
            },
            PyLayer::L3(val) => {
                Ok(Layer::Span(
                        val.into_iter().map(|(start, end, idx)| 
                            (start as usize, end as usize, Data::from_usize(idx))).collect()))
            },
            PyLayer::LS(val) => {
                let mut result = Vec::new();
                for data in val {
                    result.push(Data::from_str(data));
                }
                Ok(Layer::Seq(result))
            },
            PyLayer::L1S(val) => {
                match meta.data {
                    Some(DataType::TypedLink(_)) => {
                        let mut result = Vec::new();
                        for (idx, link) in val {
                            result.push(Data::from_link(idx, link));
                        }
                        Ok(Layer::Seq(result))
                    },
                    Some(_) => {
                        let mut result = Vec::new();
                        for (start, data) in val {
                            result.push((start, Data::from_str(data)));
                        }
                        match meta.layer_type {
                            LayerType::Div => Ok(Layer::Div(result)),
                            LayerType::Element => Ok(Layer::Element(result)),
                            _ => Err(TeangaError::ModelError(
                                format!("Cannot convert data layer to {}", meta.layer_type)))
                        }
                    },
                    None => Err(TeangaError::ModelError(
                        format!("String in data, but data type is none")))
                }
            },
            PyLayer::L2S(val) => {
                match meta.data {
                    Some(DataType::TypedLink(_)) => {
                        let mut result = Vec::new();
                        for (start, idx, link) in val {
                            result.push((start, Data::from_link(idx, link)));
                        }
                        match meta.layer_type {
                            LayerType::Div => Ok(Layer::Div(result)),
                            LayerType::Element => Ok(Layer::Element(result)),
                            _ => Err(TeangaError::ModelError(
                                format!("Cannot convert data layer to {}", meta.layer_type)))
                        }
                    },
                    _ => {
                        let mut result = Vec::new();
                        for (start, end, data) in val {
                            result.push((start, end, Data::from_str(data)));
                        }
                        Ok(Layer::Span(result))
                    }
                }
            },
            PyLayer::L3S(val) => {
                let mut result = Vec::new();
                for (start, end, idx, link) in val {
                    result.push((start, end, Data::from_link(idx, link)));
                }
                Ok(Layer::Span(result))
            },
        }
    }
}



#[derive(Error, Debug)]
pub enum TeangaError {
    //#[error("Data read error: UTF-8 String could not be decoded")]
    //UTFDataError,
    #[error("Teanga model error: {0}")]
    ModelError(String),
    #[error("Json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("UTF8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

type TeangaResult<T> = Result<T, TeangaError>;


/// Code for rendering the annotations

use yew::prelude::*;
use crate::teanga::{DocSecs, Data, Anno};

/// An iterator over a string that returns substrings of the string
/// that follow unicode code points
struct UniStrIter<'a> {
    str : &'a str,
    chars : std::str::CharIndices<'a>, 
    i : usize
}

impl<'a> UniStrIter<'a> {
    /// Create a new UniStrIter from a string slice
    fn from_str(s : &'a str) -> UniStrIter<'a> {
        let mut chars = s.char_indices();
        chars.next();
        UniStrIter { str: s, chars, i: 0 }
    }

    /// Return the next n UniCode characters as a string
    fn next(&mut self, n : usize) -> Result<&'a str, &'static str> {
        if n == 0 {
            return Ok(&self.str[self.i..self.i]);
        }
        if n > 1 {
            self.chars.nth(n - 2).ok_or("String too short")?;
        }
        if let Some((j, _)) = self.chars.next() {
            let s = &self.str[self.i..j];
            self.i = j;
            return Ok(s);
        } else if self.i < self.str.len() {
            let s = &self.str[self.i..self.str.len()];
            self.i = self.str.len();
            return Ok(s);
        } else {
            return Err("String too short");
        }
    }

    /// Get the rest of the string
    fn rest(&mut self) -> &'a str {
        let s = &self.str[self.i..];
        self.i = self.str.len();
        s
    }
}

pub fn render_annos(docsec : &DocSecs) -> Html {
    annos_to_html(&mut UniStrIter::from_str(docsec.content), &docsec.annos, 0, None)
}

fn annos_to_html(content : &mut UniStrIter, annos : &Vec<Anno>, i : usize, j : Option<usize>) -> Html {
    let mut html = Vec::new();
    let mut last_i = i;
    for anno in annos.iter() {
        if anno.start > last_i {
            let text = content.next(anno.start - last_i).unwrap();
            html.push(html! { {text} });
            last_i = anno.start;
        }
        match anno.data {
            None => html.push(html! {
                <span class="border-green-900 border-2">{ annos_to_html(content, &anno.children, last_i, Some(anno.end)) }</span>
            }),
            Some(Data::String(ref s)) => {
                html.push(html! { 
                    <ruby class="border-green-900 border-2 rounded-md">{ annos_to_html(content, &anno.children, last_i, Some(anno.end)) }
                    <rt class="bg-green-900 text-white border-2 border-green-900 rounded-t-md">{ s }</rt>
                </ruby>
                });
            },
            Some(Data::Link(ref i)) => {
                html.push(html! {
                    <ruby class="border-green-900 border-2 rounded-md">{ annos_to_html(content, &anno.children, last_i, Some(anno.end)) }
                    <rt class="bg-green-900 text-white border-2 border-green-900 rounded-t-md">{ i }</rt>
                </ruby>
                });
            },
            Some(Data::TypedLink(ref i, ref s)) => {
                html.push(html! {
                    <ruby class="border-green-900 border-2 rounded-md">{ annos_to_html(content, &anno.children, last_i, Some(anno.end)) }
                    <rt class="bg-green-900 text-white border-2 border-green-900 rounded-t-md">{ s.to_owned() + "=" + &i.to_string() }</rt>
                    </ruby>
                });
            }
        }
        last_i = anno.end;
    }
    if let Some(j) = j {
        if last_i < j {
            let text = content.next(j - last_i).unwrap();
            html.push(html! { {text} });
        }
    } else {
        html.push(html! { {content.rest()} });
    }
    html.into_iter().collect::<Html>()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unistriter() {
        let mut s = UniStrIter::from_str("This café is naïve");
        assert_eq!(s.next(4).unwrap(), "This");
        assert_eq!(s.next(5).unwrap(), " café");
        assert_eq!(s.rest(), " is naïve");
    }

    #[test]
    fn test_unistriter2() {
        let mut s = UniStrIter::from_str("This");
        assert_eq!(s.next(4).unwrap(), "This");
    }
}


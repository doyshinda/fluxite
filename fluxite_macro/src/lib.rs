use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, Expr, Token};

struct Metric {
    key: Expr,
    val: Expr,
    labels: Vec<Label>,
}

struct Label {
    key: Expr,
    val: Expr,
}

impl Parse for Metric {
    fn parse(mut input: ParseStream) -> Result<Self> {
        let key = input.parse::<Expr>()?;

        input.parse::<Token![,]>()?;
        let val = input.parse::<Expr>()?;

        let labels = parse_labels(&mut input)?;
        Ok(Metric { key, val, labels })
    }
}

#[proc_macro]
/// Emits a counter metric
pub fn count(input: TokenStream) -> TokenStream {
    let Metric { key, val, labels } = parse_macro_input!(input as Metric);

    let resp = if labels.len() > 0 {
        let r: Vec<_> = labels
            .iter()
            .map(|Label { key, val }| quote! { fluxite::Label::from_parts(&#key, &#val) })
            .collect();
        quote! {
            {
                if let Some(s) = fluxite::get_sink() {
                    let new_labels = vec![#(#r),*];
                    s.count_with_labels(#key, #val, &new_labels);
                }
            }
        }
    } else {
        quote! {
            if let Some(s) = fluxite::get_sink() {
                s.count(#key, #val);
            }
        }
    };

    resp.into()
}

fn parse_labels(input: &mut ParseStream) -> Result<Vec<Label>> {
    let mut labels: Vec<Label> = Vec::new();
    if input.is_empty() {
        return Ok(labels);
    }

    if input.peek(Token![,]) && input.peek3(Token![=>]) {
        loop {
            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }

            let k = input.parse::<Expr>()?;
            input.parse::<Token![=>]>()?;
            let v = input.parse::<Expr>()?;
            labels.push(Label { key: k, val: v });
        }

        return Ok(labels);
    }

    if input.peek(Token![,]) {
        input.parse::<Token![,]>()?;
    }

    return Ok(labels);
}

use super::ParseFieldAttr;
use quote::quote;
use syn::{ExprArray, Result};

#[derive(Debug, Default)]
pub struct Select {
    pub prompt: Option<String>,
    pub default: Option<usize>,
    pub options: Option<ExprArray>,
}

impl ParseFieldAttr for Select {
    fn parse_field_attr(attr: &syn::Attribute) -> Result<Self> {
        let mut res = Self {
            prompt: None,
            default: None,
            options: None,
        };
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("prompt") {
                meta.value()?;
                let value = meta.input.parse::<syn::LitStr>()?;
                res.prompt = Some(value.value());
                return Ok(());
            }
            if meta.path.is_ident("default") {
                meta.value()?;
                let value = meta.input.parse::<syn::LitInt>()?;
                res.default = Some(value.base10_parse()?);
                return Ok(());
            }

            if meta.path.is_ident("options") {
                meta.value()?;
                let value = meta.input.parse::<syn::ExprArray>()?;
                res.options = Some(value);
                return Ok(());
            }
            Err(meta.error("expected `prompt` , `default` or  `options`"))
        })?;
        Ok(res)
    }

    fn generate_method(
        &self,
        field_name: &Option<syn::Ident>,
        inner_ty: Option<&syn::Type>,
    ) -> Result<proc_macro2::TokenStream> {
        let mut body = proc_macro2::TokenStream::new();
        let mut params = proc_macro2::TokenStream::new();
        let mut gen_options = proc_macro2::TokenStream::new();
        // 设置主题
        if let Some(theme) = Self::get_theme() {
            body.extend(quote! {
                let res=dialogue_macro::dialoguer::Select::with_theme(#theme)
            });
        } else {
            body.extend(quote! {
                let res=dialogue_macro::dialoguer::Select::new()
            });
        }
        let Self {
            prompt,
            default,
            options,
        } = self;

        if self.prompt.is_some() {
            body.extend(quote!(
                .with_prompt(#prompt)
            ))
        } else {
            params.extend(quote! {
                prompt: &str,
            });
            body.extend(quote!(
                .with_prompt(prompt)
            ))
        }

        if default.is_some() {
            body.extend(quote!(
                .default(#default)
            ))
        }

        if options.is_some() {
            gen_options.extend(quote!(
                let options:&[#inner_ty]=&vec!#options;
            ));
        } else {
            params.extend(quote! {
                options: &[#inner_ty],
            });
        }
        body.extend(quote!(
            .items(options)
        ));

        Ok(quote! {
            pub fn #field_name(&mut self,#params) -> &mut Self{
                #gen_options
                #body.interact().unwrap();
                self.#field_name=Some(options[res].clone().into());
                self
            }
        })
    }
}

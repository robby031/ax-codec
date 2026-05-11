use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields, Index, parse_macro_input, parse_quote};

#[proc_macro_derive(Encode, attributes(axcodec))]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let encode_body = match &input.data {
        Data::Struct(data) => struct_encode_body(&data.fields),
        Data::Enum(data) => enum_encode_body(name, data),
        Data::Union(_) => {
            return syn::Error::new_spanned(input.ident, "Encode does not support unions")
                .to_compile_error()
                .into();
        }
    };

    let expanded = quote! {
        impl #impl_generics axcodec_core::Encode for #name #ty_generics #where_clause {
            #[inline]
            fn encode<W: axcodec_core::BufferWriter>(&self, writer: &mut W) -> Result<(), axcodec_core::EncodeError> {
                #encode_body
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(View, attributes(axcodec))]
pub fn derive_view(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let mut view_generics = input.generics.clone();
    view_generics.params.insert(0, parse_quote!('__a));
    let (impl_generics, _ty_generics, where_clause) = view_generics.split_for_impl();

    let mut target_generics = input.generics.clone();
    for param in &mut target_generics.params {
        if let syn::GenericParam::Lifetime(ref mut l) = *param {
            l.lifetime.ident = syn::Ident::new("__a", l.lifetime.ident.span());
        }
    }
    let (_, target_ty_generics, _) = target_generics.split_for_impl();

    let view_body = match &input.data {
        Data::Struct(data) => struct_view_body(&data.fields, name),
        Data::Enum(data) => enum_view_body(name, data),
        Data::Union(_) => {
            return syn::Error::new_spanned(input.ident, "View does not support unions")
                .to_compile_error()
                .into();
        }
    };

    let expanded = quote! {
        impl #impl_generics axcodec_core::View<'__a> for #name #target_ty_generics #where_clause {
            #[inline]
            fn view<R: axcodec_core::BufferReader<'__a>>(reader: &mut R) -> Result<Self, axcodec_core::DecodeError> {
                #view_body
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(Decode, attributes(axcodec))]
pub fn derive_decode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let decode_body = match &input.data {
        Data::Struct(data) => struct_decode_body(&data.fields, name),
        Data::Enum(data) => enum_decode_body(name, data),
        Data::Union(_) => {
            return syn::Error::new_spanned(input.ident, "Decode does not support unions")
                .to_compile_error()
                .into();
        }
    };

    let expanded = quote! {
        impl #impl_generics axcodec_core::Decode for #name #ty_generics #where_clause {
            #[inline]
            fn decode<'__a, R: axcodec_core::BufferReader<'__a>>(reader: &mut R) -> Result<Self, axcodec_core::DecodeError> {
                #decode_body
            }
        }
    };

    TokenStream::from(expanded)
}

fn struct_encode_body(fields: &Fields) -> proc_macro2::TokenStream {
    match fields {
        Fields::Named(fields) => {
            let calls = fields
                .named
                .iter()
                .filter(|f| !has_skip_attr(&f.attrs))
                .map(|f| {
                    let ident = &f.ident;
                    quote! {
                        axcodec_core::Encode::encode(&self.#ident, writer)?;
                    }
                });
            quote! { #(#calls)* }
        }
        Fields::Unnamed(fields) => {
            let calls = fields
                .unnamed
                .iter()
                .enumerate()
                .filter(|(_, f)| !has_skip_attr(&f.attrs))
                .map(|(i, _)| {
                    let index = Index::from(i);
                    quote! {
                        axcodec_core::Encode::encode(&self.#index, writer)?;
                    }
                });
            quote! { #(#calls)* }
        }
        Fields::Unit => quote! {},
    }
}

fn struct_decode_body(fields: &Fields, name: &syn::Ident) -> proc_macro2::TokenStream {
    let body = match fields {
        Fields::Named(fields) => {
            let idents: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
            let decodes = fields.named.iter().map(|f| {
                let ident = &f.ident;
                let ty = &f.ty;
                if has_skip_attr(&f.attrs) {
                    quote! {
                        let #ident: #ty = Default::default();
                    }
                } else if has_default_attr(&f.attrs) {
                    quote! {
                        let #ident: #ty = match axcodec_core::Decode::decode(reader) {
                            Ok(val) => val,
                            Err(axcodec_core::DecodeError::UnexpectedEOF) => Default::default(),
                            Err(e) => return Err(e),
                        };
                    }
                } else {
                    quote! {
                        let #ident: #ty = axcodec_core::Decode::decode(reader)?;
                    }
                }
            });
            quote! {
                #(#decodes)*
                Ok(#name { #(#idents),* })
            }
        }
        Fields::Unnamed(fields) => {
            let decodes = fields.unnamed.iter().map(|f| {
                let ty = &f.ty;
                if has_skip_attr(&f.attrs) {
                    quote! {
                        <#ty as Default>::default()
                    }
                } else if has_default_attr(&f.attrs) {
                    quote! {
                        match <#ty as axcodec_core::Decode>::decode(reader) {
                            Ok(v) => v,
                            Err(axcodec_core::DecodeError::UnexpectedEOF) => <#ty as Default>::default(),
                            Err(e) => return Err(e),
                        }
                    }
                } else {
                    quote! {
                        <#ty as axcodec_core::Decode>::decode(reader)?
                    }
                }
            });
            quote! {
                Ok(#name(#(#decodes),*))
            }
        }
        Fields::Unit => quote! { Ok(#name) },
    };
    quote! {
        {
            reader.depth_enter()?;
            let __res = (|| { #body })();
            reader.depth_exit();
            __res
        }
    }
}

fn struct_view_body(fields: &Fields, name: &syn::Ident) -> proc_macro2::TokenStream {
    let body = match fields {
        Fields::Named(fields) => {
            let idents: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
            let decodes = fields.named.iter().map(|f| {
                let ident = &f.ident;
                let ty = &f.ty;
                if has_skip_attr(&f.attrs) {
                    quote! {
                        let #ident: #ty = Default::default();
                    }
                } else {
                    quote! {
                        let #ident = axcodec_core::View::view(reader)?;
                    }
                }
            });
            quote! {
                #(#decodes)*
                Ok(#name { #(#idents),* })
            }
        }
        Fields::Unnamed(fields) => {
            let decodes = fields.unnamed.iter().map(|f| {
                let ty = &f.ty;
                if has_skip_attr(&f.attrs) {
                    quote! {
                        <#ty as Default>::default()
                    }
                } else {
                    quote! {
                        <#ty as axcodec_core::View>::view(reader)?
                    }
                }
            });
            quote! {
                Ok(#name(#(#decodes),*))
            }
        }
        Fields::Unit => quote! { Ok(#name) },
    };
    quote! {
        {
            reader.depth_enter()?;
            let __res = (|| { #body })();
            reader.depth_exit();
            __res
        }
    }
}

fn enum_encode_body(name: &syn::Ident, data: &syn::DataEnum) -> proc_macro2::TokenStream {
    let discriminant_ty = if data.variants.len() <= 256 {
        quote! { u8 }
    } else {
        quote! { u16 }
    };

    let arms = data.variants.iter().enumerate().map(|(idx, variant)| {
        let variant_name = &variant.ident;
        let idx_lit = if data.variants.len() <= 256 {
            let idx_u8 = idx as u8;
            quote! { #idx_u8 }
        } else {
            let idx_u16 = idx as u16;
            quote! { #idx_u16 }
        };

        match &variant.fields {
            Fields::Named(fields) => {
                let idents: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                let encodes = fields
                    .named
                    .iter()
                    .filter(|f| !has_skip_attr(&f.attrs))
                    .map(|f| {
                        let ident = &f.ident;
                        quote! {
                            axcodec_core::Encode::encode(#ident, writer)?;
                        }
                    });
                quote! {
                    #name::#variant_name { #(#idents),* } => {
                        (#idx_lit as #discriminant_ty).encode(writer)?;
                        #(#encodes)*
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let idents: Vec<_> = (0..fields.unnamed.len())
                    .map(|i| syn::Ident::new(&format!("__field_{}", i), Span::call_site()))
                    .collect();
                let encodes = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .filter(|(_, f)| !has_skip_attr(&f.attrs))
                    .map(|(i, _)| {
                        let ident = &idents[i];
                        quote! {
                            axcodec_core::Encode::encode(#ident, writer)?;
                        }
                    });
                quote! {
                    #name::#variant_name(#(#idents),*) => {
                        (#idx_lit as #discriminant_ty).encode(writer)?;
                        #(#encodes)*
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    #name::#variant_name => {
                        (#idx_lit as #discriminant_ty).encode(writer)?;
                    }
                }
            }
        }
    });

    quote! {
        match self {
            #(#arms)*
        }
    }
}

fn enum_decode_body(name: &syn::Ident, data: &syn::DataEnum) -> proc_macro2::TokenStream {
    let discriminant_ty = if data.variants.len() <= 256 {
        quote! { u8 }
    } else {
        quote! { u16 }
    };

    let arms = data.variants.iter().enumerate().map(|(idx, variant)| {
        let variant_name = &variant.ident;
        let idx_lit = if data.variants.len() <= 256 {
            let idx_u8 = idx as u8;
            quote! { #idx_u8 }
        } else {
            let idx_u16 = idx as u16;
            quote! { #idx_u16 }
        };

        match &variant.fields {
            Fields::Named(fields) => {
                let idents: Vec<_> = fields
                    .named
                    .iter()
                    .filter(|f| !has_skip_attr(&f.attrs))
                    .map(|f| &f.ident)
                    .collect();
                let decodes = fields.named.iter().map(|f| {
                    let ident = &f.ident;
                    let ty = &f.ty;
                    if has_skip_attr(&f.attrs) {
                        quote! {
                            let #ident: #ty = Default::default();
                        }
                    } else {
                        quote! {
                            let #ident: #ty = axcodec_core::Decode::decode(reader)?;
                        }
                    }
                });
                quote! {
                    #idx_lit => {
                        #(#decodes)*
                        Ok(#name::#variant_name { #(#idents),* })
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let decodes = fields.unnamed.iter().map(|f| {
                    let ty = &f.ty;
                    if has_skip_attr(&f.attrs) {
                        quote! {
                            <#ty as Default>::default()
                        }
                    } else {
                        quote! {
                            <#ty as axcodec_core::Decode>::decode(reader)?
                        }
                    }
                });
                quote! {
                    #idx_lit => Ok(#name::#variant_name(#(#decodes),*)),
                }
            }
            Fields::Unit => {
                quote! {
                    #idx_lit => Ok(#name::#variant_name),
                }
            }
        }
    });

    quote! {
        {
            reader.depth_enter()?;
            let __res = (|| {
                let discriminant: #discriminant_ty = axcodec_core::Decode::decode(reader)?;
                match discriminant {
                    #(#arms)*
                    _ => Err(axcodec_core::DecodeError::UnexpectedEOF),
                }
            })();
            reader.depth_exit();
            __res
        }
    }
}

fn enum_view_body(name: &syn::Ident, data: &syn::DataEnum) -> proc_macro2::TokenStream {
    let discriminant_ty = if data.variants.len() <= 256 {
        quote! { u8 }
    } else {
        quote! { u16 }
    };

    let arms = data.variants.iter().enumerate().map(|(idx, variant)| {
        let variant_name = &variant.ident;
        let idx_lit = if data.variants.len() <= 256 {
            let idx_u8 = idx as u8;
            quote! { #idx_u8 }
        } else {
            let idx_u16 = idx as u16;
            quote! { #idx_u16 }
        };

        match &variant.fields {
            Fields::Named(fields) => {
                let idents: Vec<_> = fields
                    .named
                    .iter()
                    .filter(|f| !has_skip_attr(&f.attrs))
                    .map(|f| &f.ident)
                    .collect();
                let decodes = fields.named.iter().map(|f| {
                    let ident = &f.ident;
                    let ty = &f.ty;
                    if has_skip_attr(&f.attrs) {
                        quote! {
                            let #ident: #ty = Default::default();
                        }
                    } else {
                        quote! {
                            let #ident: #ty = axcodec_core::View::view(reader)?;
                        }
                    }
                });
                quote! {
                    #idx_lit => {
                        #(#decodes)*
                        Ok(#name::#variant_name { #(#idents),* })
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let decodes = fields.unnamed.iter().map(|f| {
                    let ty = &f.ty;
                    if has_skip_attr(&f.attrs) {
                        quote! {
                            <#ty as Default>::default()
                        }
                    } else {
                        quote! {
                            <#ty as axcodec_core::View>::view(reader)?
                        }
                    }
                });
                quote! {
                    #idx_lit => Ok(#name::#variant_name(#(#decodes),*)),
                }
            }
            Fields::Unit => {
                quote! {
                    #idx_lit => Ok(#name::#variant_name),
                }
            }
        }
    });

    quote! {
        {
            reader.depth_enter()?;
            let __res = (|| {
                let discriminant: #discriminant_ty = axcodec_core::Decode::decode(reader)?;
                match discriminant {
                    #(#arms)*
                    _ => Err(axcodec_core::DecodeError::UnexpectedEOF),
                }
            })();
            reader.depth_exit();
            __res
        }
    }
}

fn has_skip_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("axcodec")
            && attr
                .parse_args::<syn::Ident>()
                .is_ok_and(|ident| ident == "skip")
    })
}

fn has_default_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("axcodec")
            && attr
                .parse_args::<syn::Ident>()
                .is_ok_and(|ident| ident == "default")
    })
}

use heck::SnekCase;
use proc_macro::TokenStream;
use quote::*;
use syn::{parse::*, punctuated::*, *};

mod kw {
    syn::custom_keyword!(components);
    syn::custom_keyword!(resources);

    syn::custom_keyword!(read);
    syn::custom_keyword!(write);

    syn::custom_keyword!(read_expect);
    syn::custom_keyword!(write_expect);

    syn::custom_keyword!(read_storage);
    syn::custom_keyword!(write_storage);

    syn::custom_keyword!(entities);
}

struct Values {
    values: Punctuated<Ident, Token![,]>,
}

impl Parse for Values {
    fn parse(input: ParseStream) -> Result<Self> {
        let values = group::parse_parens(input)?.content;
        Ok(Values {
            values: values.parse_terminated(Ident::parse)?,
        })
    }
}

impl Values {
    fn iter(&self) -> punctuated::Iter<Ident> {
        self.values.iter()
    }
}

struct SaveloadInputStruct {
    components: Values,
    resources: Values,
}

impl Parse for SaveloadInputStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<kw::components>()?;
        let components = input.parse()?;
        input.parse::<kw::resources>()?;
        let resources = input.parse()?;
        Ok(SaveloadInputStruct {
            components,
            resources,
        })
    }
}

fn chunk_components(input: Vec<Ident>) -> Vec<Vec<Ident>> {
    input.chunks(16).map(Vec::from).collect()
}

#[proc_macro]
pub fn saveload_system_data(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as SaveloadInputStruct);

    // Prepare resources
    let resource_types: Vec<Ident> = parsed.resources.iter().cloned().collect();
    let resource_names: Vec<Ident> = resource_types
        .iter()
        .cloned()
        .map(|ident| syn::Ident::new(&ident.to_string().to_snek_case(), ident.span()))
        .collect();

    // Chunk components into tuples of at most 16 items, as that's the limit of what
    // specs_derive defines
    let component_types: Vec<Ident> = parsed.components.iter().cloned().collect();
    let component_chunks = chunk_components(component_types);
    let component_chunk_count = component_chunks.len();
    let chunk_ids = 0..component_chunk_count;
    let chunk_indexes: Vec<Index> = chunk_ids.clone().map(Index::from).collect();

    // Prepare for SaveSystemData
    let read_component_tuples: Vec<_> = component_chunks
        .iter()
        .map(|chunk| quote! { (#(ReadStorage<'a, #chunk>,)*) })
        .collect();
    let read_components_tuple = quote! { (#(#read_component_tuples,)*) };
    let ser_fns: Vec<Ident> = chunk_ids
        .clone()
        .map(|n| format_ident!("ser_{}", n))
        .collect();

    // Prepare for LoadSystemData
    let write_component_tuples: Vec<_> = component_chunks
        .iter()
        .map(|chunk| quote! { (#(WriteStorage<'a, #chunk>,)*) })
        .collect();
    let write_components_tuple = quote! { (#(#write_component_tuples,)*) };
    let deser_fns: Vec<Ident> = chunk_ids.map(|n| format_ident!("deser_{}", n)).collect();

    // And build the thing
    let expanded = quote! {
        type SaveSystemDataComponents<'a> = #read_components_tuple;

        #[derive(SystemData)]
        pub struct SaveSystemData<'a> {
            entities: Entities<'a>,

            markers: ReadStorage<'a, SimpleMarker<SerializeMe>>,
            marker_alloc: SpecsWrite<'a, SimpleMarkerAllocator<SerializeMe>>,
            components: SaveSystemDataComponents<'a>,

            #(
            #resource_names: ReadExpect<'a, #resource_types>,
            )*
        }

        impl<'a> SaveSystemData<'a> {
            #(
            fn #ser_fns<S>(&self, serializer: S) where S: Serializer {
                SerializeComponents::<NoError, _>::serialize(
                    &self.components.#chunk_indexes,
                    &self.entities,
                    &self.markers,
                    serializer,
                ).expect("Serialization failed");
            }
            )*

            fn ser<W>(&self, mut serializer: ron::Serializer<W>) where W: Write {
                #(
                    self.#ser_fns(&mut serializer);
                )*
            }
        }

        type LoadSystemDataComponents<'a> = #write_components_tuple;

        #[derive(SystemData)]
        pub struct LoadSystemData<'a> {
            entities: Entities<'a>,

            markers: WriteStorage<'a, SimpleMarker<SerializeMe>>,
            marker_alloc: SpecsWrite<'a, SimpleMarkerAllocator<SerializeMe>>,
            components: LoadSystemDataComponents<'a>,

            #(
            #resource_names: WriteExpect<'a, #resource_types>,
            )*
        }

        impl<'a> LoadSystemData<'a> {
            #(
            fn #deser_fns<'de, D>(&mut self, deserializer: D) where D: Deserializer<'de> {
                DeserializeComponents::<NoError, _>::deserialize(
                    &mut self.components.#chunk_indexes,
                    &self.entities,
                    &mut self.markers,
                    &mut self.marker_alloc,
                    deserializer,
                ).expect("Deserialization failed");
            }
            )*

            fn deser(&mut self, mut deserializer: ron::Deserializer) {
                #(
                    self.#deser_fns(&mut deserializer);
                )*
            }
        }
    };

    TokenStream::from(expanded)
}

#[derive(Debug)]
struct SystemdataInput {
    name: Ident,
    read: Vec<Ident>,
    write: Vec<Ident>,
    read_expect: Vec<Ident>,
    write_expect: Vec<Ident>,
    read_storage: Vec<Ident>,
    write_storage: Vec<Ident>,
    entities: Option<Ident>,
}

impl Parse for SystemdataInput {
    fn parse(input: &ParseBuffer) -> Result<Self> {
        let name = input.parse::<Ident>()?;
        let inner: ParseBuffer;
        parenthesized!(inner in input);

        let mut read: Vec<Ident> = vec![];
        let mut write: Vec<Ident> = vec![];
        let mut read_expect: Vec<Ident> = vec![];
        let mut write_expect: Vec<Ident> = vec![];
        let mut read_storage: Vec<Ident> = vec![];
        let mut write_storage: Vec<Ident> = vec![];
        let mut entities: Option<Ident> = None;

        while !inner.is_empty() {
            let lookahead = inner.lookahead1();
            if lookahead.peek(kw::entities) {
                entities = Some(inner.parse()?);
            } else if lookahead.peek(Token![,]) {
                inner.parse::<Token![,]>()?;
            } else {
                let target = if lookahead.peek(kw::read) {
                    inner.parse::<kw::read>()?;
                    &mut read
                } else if lookahead.peek(kw::write) {
                    inner.parse::<kw::write>()?;
                    &mut write
                } else if lookahead.peek(kw::read_expect) {
                    inner.parse::<kw::read_expect>()?;
                    &mut read_expect
                } else if lookahead.peek(kw::write_expect) {
                    inner.parse::<kw::write_expect>()?;
                    &mut write_expect
                } else if lookahead.peek(kw::read_storage) {
                    inner.parse::<kw::read_storage>()?;
                    &mut read_storage
                } else if lookahead.peek(kw::write_storage) {
                    inner.parse::<kw::write_storage>()?;
                    &mut write_storage
                } else {
                    return Err(lookahead.error());
                };
                let content;
                parenthesized!(content in inner);
                let values: Punctuated<Ident, Token![,]> =
                    content.parse_terminated(Ident::parse)?;
                let mut idents = values.iter().cloned().collect();
                target.append(&mut idents);
            }
        }

        Ok(SystemdataInput {
            name,
            read,
            write,
            read_expect,
            write_expect,
            read_storage,
            write_storage,
            entities,
        })
    }
}

fn component_names(component_types: &[Ident]) -> Vec<Ident> {
    component_types
        .iter()
        .map(|ident| {
            let snek = ident.to_string().to_snek_case();
            let suffix = if snek.ends_with('s') { "es" } else { "s" };
            syn::Ident::new(&format!("{}{}", snek, suffix), ident.span())
        })
        .collect()
}

fn resource_names(resource_types: &[Ident]) -> Vec<Ident> {
    resource_types
        .iter()
        .map(|ident| syn::Ident::new(&ident.to_string().to_snek_case(), ident.span()))
        .collect()
}

#[proc_macro]
pub fn systemdata(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as SystemdataInput);

    let name = parsed.name;

    let entities: Vec<Ident> = if let Some(ident) = parsed.entities {
        vec![ident]
    } else {
        vec![]
    };

    let read_storage_types = parsed.read_storage;
    let read_storage_names = component_names(&read_storage_types);

    let write_storage_types = parsed.write_storage;
    let write_storage_names = component_names(&write_storage_types);

    let read_types = parsed.read;
    let read_names = resource_names(&read_types);

    let write_types = parsed.write;
    let write_names = resource_names(&write_types);

    let read_expect_types = parsed.read_expect;
    let read_expect_names = resource_names(&read_expect_types);

    let write_expect_types = parsed.write_expect;
    let write_expect_names = resource_names(&write_expect_types);

    let expanded = quote! {
        use shred_derive::SystemData;
        use crate::components::{#(#read_storage_types, )* #(#write_storage_types, )*};
        use crate::resources::{#(#read_types, )* #(#write_types, )* #(#read_expect_types, )* #(#write_expect_types, )*};

        #[derive(SystemData)]
        pub struct #name<'a> {
            #(#entities: Entities<'a>,)*

            #(#read_storage_names: ReadStorage<'a, #read_storage_types>,)*
            #(#write_storage_names: WriteStorage<'a, #write_storage_types>,)*

            #(#read_names: Read<'a, #read_types>,)*
            #(#write_names: Write<'a, #write_types>,)*

            #(#read_expect_names: ReadExpect<'a, #read_expect_types>,)*
            #(#write_expect_names: WriteExpect<'a, #write_expect_types>,)*
        }
    };

    TokenStream::from(expanded)
}

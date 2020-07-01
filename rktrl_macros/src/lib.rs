use heck::SnekCase;
use proc_macro::TokenStream;
use quote::*;
use syn::{parse::*, punctuated::*, *};

mod kw {
    syn::custom_keyword!(components);
    syn::custom_keyword!(resources);
}

struct InputStruct {
    _components_token: kw::components,
    _components_paren: token::Paren,
    components: Punctuated<Ident, Token![,]>,

    _resources_token: kw::resources,
    _resources_paren: token::Paren,
    resources: Punctuated<Ident, Token![,]>,
}

impl Parse for InputStruct {
    #[allow(clippy::eval_order_dependence)]
    fn parse(input: ParseStream) -> Result<Self> {
        let components;
        let resources;
        Ok(InputStruct {
            _components_token: input.parse()?,
            _components_paren: parenthesized!(components in input),
            components: components.parse_terminated(Ident::parse)?,

            _resources_token: input.parse()?,
            _resources_paren: parenthesized!(resources in input),
            resources: resources.parse_terminated(Ident::parse)?,
        })
    }
}

#[proc_macro]
pub fn saveload_system_data(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as InputStruct);

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
    let component_chunks: Vec<Vec<Ident>> = component_types.chunks(16).map(Vec::from).collect();
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
            marker_alloc: Write<'a, SimpleMarkerAllocator<SerializeMe>>,
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

            fn ser<W>(&self, mut serializer: ron::Serializer<W>) where W: std::io::Write {
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
            marker_alloc: Write<'a, SimpleMarkerAllocator<SerializeMe>>,
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

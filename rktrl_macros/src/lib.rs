use proc_macro::TokenStream;
use quote::*;
use syn::{parse::*, punctuated::*, *};

#[proc_macro]
pub fn save_system_data(input: TokenStream) -> TokenStream {
    let parser = Punctuated::<Ident, Token![,]>::parse_terminated;

    let component_types: Vec<Ident> = parser.parse(input).unwrap().iter().cloned().collect();
    let component_chunks: Vec<Vec<Ident>> = component_types.chunks(8).map(Vec::from).collect();
    let component_tuples: Vec<_> = component_chunks
        .iter()
        .map(|chunk| quote! { (#(ReadStorage<'a, #chunk>,)*) })
        .collect();
    let components_tuple = quote! { (#(#component_tuples,)*) };
    let component_chunk_count = component_chunks.len();

    let chunk_ids = 0..component_chunk_count;
    let chunk_indexes: Vec<Index> = chunk_ids.clone().map(Index::from).collect();
    let ser_fns: Vec<Ident> = chunk_ids.map(|n| format_ident!("ser_{}", n)).collect();

    let expanded = quote! {
        type SaveSystemDataComponents<'a> = #components_tuple;

        #[derive(SystemData)]
        pub struct SaveSystemData<'a> {
            entities: Entities<'a>,
            markers: ReadStorage<'a, SimpleMarker<SerializeMe>>,
            components: SaveSystemDataComponents<'a>
        }

        impl<'a> SaveSystemData<'a> {
            #(
            fn #ser_fns<S>(&self, serializer: S) where S: Serializer {
                SerializeComponents::<NoError, SimpleMarker<SerializeMe>>::serialize(
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
    };

    TokenStream::from(expanded)
}

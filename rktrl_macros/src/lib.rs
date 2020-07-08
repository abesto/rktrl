use heck::SnekCase;
use proc_macro::TokenStream;
use quote::*;
use syn::{parse::*, *};

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

#[derive(Clone, Debug)]
struct ValueItem {
    ident: Option<Ident>,
    ty: Type,
}

impl ValueItem {
    fn component_name(&self) -> Ident {
        self.ident.as_ref().cloned().unwrap_or_else(|| {
            let snek = type_to_string(&self.ty).to_snek_case();
            let suffix = if snek.ends_with('s') { "es" } else { "s" };
            format_ident!("{}{}", snek, suffix)
        })
    }

    fn resource_name(&self) -> Ident {
        self.ident
            .as_ref()
            .cloned()
            .unwrap_or_else(|| format_ident!("{}", type_to_string(&self.ty).to_snek_case()))
    }
}

impl Parse for ValueItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(token::Paren) {
            let inner = group::parse_parens(input)?.content;
            let name = Some(inner.parse()?);
            inner.parse::<Token![:]>()?;
            let ty = inner.parse()?;
            Ok(ValueItem { ident: name, ty })
        } else {
            Ok(ValueItem {
                ident: None,
                ty: input.parse()?,
            })
        }
    }
}

#[derive(Debug, Clone)]
struct Values {
    items: Vec<ValueItem>,
}

impl Parse for Values {
    fn parse(input: ParseStream) -> Result<Self> {
        let inner = group::parse_parens(input)?.content;
        let items = inner
            .parse_terminated::<ValueItem, Token![,]>(ValueItem::parse)?
            .iter()
            .cloned()
            .collect();
        Ok(Values { items })
    }
}

impl Values {
    fn new() -> Self {
        Values { items: vec![] }
    }

    fn append(&mut self, other: &mut Values) {
        self.items.append(&mut other.items);
    }

    fn iter(&self) -> std::slice::Iter<ValueItem> {
        self.items.iter()
    }

    fn types(&self) -> Vec<Type> {
        self.iter().map(|item| item.ty.clone()).collect()
    }

    fn component_names(&self) -> Vec<Ident> {
        self.iter().map(|item| item.component_name()).collect()
    }

    fn resource_names(&self) -> Vec<Ident> {
        self.iter().map(|item| item.resource_name()).collect()
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

fn chunk_components(input: Vec<Type>) -> Vec<Vec<Type>> {
    input.chunks(16).map(Vec::from).collect()
}

#[proc_macro]
pub fn saveload_system_data(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as SaveloadInputStruct);

    // Prepare resources
    let resource_types = parsed.resources.types();
    let resource_names = parsed.resources.resource_names();

    // Chunk components into tuples of at most 16 items, as that's the limit of what
    // specs_derive defines
    let component_types = parsed.components.types();
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
    read: Values,
    write: Values,
    read_expect: Values,
    write_expect: Values,
    read_storage: Values,
    write_storage: Values,
    entities: bool,
}

impl Parse for SystemdataInput {
    fn parse(input: &ParseBuffer) -> Result<Self> {
        let name = input.parse::<Ident>()?;
        let inner = group::parse_parens(input)?.content;

        let mut read = Values::new();
        let mut write = Values::new();
        let mut read_expect = Values::new();
        let mut write_expect = Values::new();
        let mut read_storage = Values::new();
        let mut write_storage = Values::new();
        let mut entities = false;

        while !inner.is_empty() {
            let lookahead = inner.lookahead1();
            if lookahead.peek(kw::entities) {
                inner.parse::<kw::entities>()?;
                entities = true;
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
                let mut values = inner.parse::<Values>()?;
                target.append(&mut values);
            }
            if !inner.is_empty() {
                inner.parse::<Token![,]>()?;
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

fn type_to_string(t: &Type) -> String {
    format!("{}", t.to_token_stream())
}

#[proc_macro]
pub fn systemdata(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as SystemdataInput);

    let name = parsed.name;

    let entities = if parsed.entities {
        vec![format_ident!("{}", "entities")]
    } else {
        vec![]
    };

    let read_storage_types = parsed.read_storage.types();
    let read_storage_names = parsed.read_storage.component_names();

    let write_storage_types = parsed.write_storage.types();
    let write_storage_names = parsed.write_storage.component_names();

    let read_types = parsed.read.types();
    let read_names = parsed.read.resource_names();

    let write_types = parsed.write.types();
    let write_names = parsed.write.resource_names();

    let read_expect_types = parsed.read_expect.types();
    let read_expect_names = parsed.read_expect.resource_names();

    let write_expect_types = parsed.write_expect.types();
    let write_expect_names = parsed.write_expect.resource_names();

    let expanded = quote! {
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

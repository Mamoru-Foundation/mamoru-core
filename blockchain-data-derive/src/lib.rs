/// This macro automatically implements [`mamoru_core::BlockchainData`] trait:
/// - puts a virtual table name
/// - generates Apache Arrow Schema
/// - generates a code to convert a list of the struct instances to RecordBatch
use darling::{ast, util, FromDeriveInput, FromField, FromMeta};
use maplit::hashmap;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput, Expr};

#[proc_macro_derive(BlockchainData, attributes(schema))]
pub fn derive_blockchain_data(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let data = StructInfo::from_derive_input(&input).unwrap();

    DeriveImpl::new(data)
        .generate()
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[derive(FromDeriveInput)]
#[darling(attributes(schema), supports(struct_named))]
struct StructInfo {
    ident: Ident,
    data: ast::Data<util::Ignored, Field>,
    table_name: String,

    #[darling(default)]
    mamoru_path: Option<Expr>,
}

#[derive(FromField, Debug, Clone)]
#[darling(attributes(schema))]
struct Field {
    ident: Option<Ident>,

    #[darling(rename = "type")]
    arrow_type: Expr,

    #[darling(default)]
    rename: Option<String>,

    #[darling(default)]
    nullable: bool,
}

impl Field {
    fn ident(&self) -> &Ident {
        self.ident
            .as_ref()
            .expect("BUG: the input is always struct.")
    }
}

struct DeriveImpl {
    info: StructInfo,
    builders: Builders,
    arrow_path: Expr,
}

impl DeriveImpl {
    pub(crate) fn new(info: StructInfo) -> Self {
        let arrow_path = as_expr("::datafusion::arrow");
        let builders = Builders::new(&arrow_path);

        Self {
            info,
            builders,
            arrow_path,
        }
    }

    pub(crate) fn generate(&self) -> syn::Result<TokenStream> {
        let struct_name = &self.info.ident;
        let batch_struct_name = format_ident!("{}Batch", &struct_name);

        let table_name = &self.info.table_name;
        let schema = self.generate_schema()?;
        let to_record_batch = self.generate_to_record_batch()?;
        let mamoru_path = self
            .info
            .mamoru_path
            .clone()
            .unwrap_or_else(|| as_expr("mamoru_core"));

        Ok(quote! {
            pub struct #batch_struct_name(pub Vec<#struct_name>);

            impl #batch_struct_name {
                pub fn new(items: impl IntoIterator<Item = #struct_name>) -> Self {
                    Self(items.into_iter().collect())
                }

                pub fn boxed(self) -> Box<Self> {
                    Box::new(self)
                }
            }

            impl #mamoru_path::BlockchainData for #batch_struct_name {
                fn table_name(&self) -> &'static str {
                    #table_name
                }

                #schema
                #to_record_batch
            }
        })
    }

    fn generate_to_record_batch(&self) -> syn::Result<TokenStream> {
        let arrow = &self.arrow_path;

        let mut init_builders = vec![];
        let mut appends = vec![];
        let mut finish_builders = vec![];

        for field in self.fields() {
            let ident = field.ident();

            let builder_name = format_ident!("{}_builder", ident);
            let builder_constructor = self.builders.find(&field.arrow_type)?;

            init_builders.push(quote! {
                let mut #builder_name = (#builder_constructor)(len);
            });

            appends.push(if field.nullable {
                quote! {
                    #builder_name.append_option(item.#ident);
                }
            } else {
                quote! {
                    #builder_name.append_value(item.#ident);
                }
            });

            finish_builders.push(quote! {
                std::sync::Arc::new(#builder_name.finish())
            })
        }

        Ok(quote! {
            fn to_record_batch(self: Box<Self>) -> Result<#arrow::record_batch::RecordBatch, #arrow::error::ArrowError> {
                let len = self.0.len();
                let schema = self.schema();

                #(#init_builders)*

                for item in self.0 {
                    #(#appends)*
                }

                let batch = #arrow::record_batch::RecordBatch::try_new(schema, vec![#(#finish_builders,)*])?;

                Ok(batch)
            }
        })
    }

    fn generate_schema(&self) -> syn::Result<TokenStream> {
        let arrow = &self.arrow_path;

        let schema_fields: Vec<TokenStream> = self
            .fields()
            .iter()
            .map(|field| {
                let (arrow_type, nullable) = (&field.arrow_type, field.nullable);

                let name = match &field.rename {
                    Some(name) => name.clone(),
                    None => field
                        .ident
                        .as_ref()
                        .map(|ident| ident.to_string())
                        .expect("BUG: the input is always struct."),
                };

                quote! { #arrow::datatypes::Field::new(#name, #arrow_type, #nullable) }
            })
            .collect();

        Ok(quote! {
             fn schema(&self) -> ::std::sync::Arc<#arrow::datatypes::Schema> {
                let schema = #arrow::datatypes::Schema::new(vec![
                    #(#schema_fields,)*
                ]);

                ::std::sync::Arc::new(schema)
            }
        })
    }

    fn fields(&self) -> &[Field] {
        match &self.info.data {
            ast::Data::Struct(fields) => &fields.fields,
            ast::Data::Enum(_) => panic!("BUG: the input is always struct."),
        }
    }
}

struct Builders {
    map: HashMap<Expr, TokenStream>,
}

impl Builders {
    fn new(arrow: &Expr) -> Self {
        let map = hashmap! {
            as_expr("DataType::Binary") => quote!{ |len: usize| { #arrow::array::BinaryBuilder::with_capacity(len, len * 32) } },
            as_expr("DataType::Utf8") => quote!{ |len: usize| { #arrow::array::StringBuilder::with_capacity(len, len * 32) } },
            as_expr("DataType::UInt64") => quote!{ #arrow::array::PrimitiveBuilder::<#arrow::datatypes::UInt64Type>::with_capacity },
            as_expr("DataType::UInt32") => quote!{ #arrow::array::PrimitiveBuilder::<#arrow::datatypes::UInt32Type>::with_capacity },
            as_expr("DataType::UInt8") => quote!{ #arrow::array::PrimitiveBuilder::<#arrow::datatypes::UInt8Type>::with_capacity },
            as_expr("DataType::Timestamp(TimeUnit::Second, None)") => quote!{ #arrow::array::PrimitiveBuilder::<#arrow::datatypes::TimestampSecondType>::with_capacity },
        };

        Self { map }
    }

    fn find(&self, typ: &Expr) -> syn::Result<TokenStream> {
        if let Some(init) = self.map.get(typ) {
            Ok(init.clone())
        } else {
            Err(syn::Error::new(
                typ.span(),
                format!("No builder found for {:?}.", typ),
            ))
        }
    }
}

fn as_expr(expr: &str) -> Expr {
    Expr::from_string(expr).expect("BUG: Failed to parse expression in macros.")
}

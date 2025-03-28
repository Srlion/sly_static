use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{parse_macro_input, parse_str, punctuated::Punctuated, Ident, ItemStatic, Token};

fn sly_static_functions_ident() -> Ident {
	Ident::new("__SLY_STATIC_FUNCTIONS", Span::call_site().into())
}

// Copyright (c) [2024] [mmastrac/rust-ctor]
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
// INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
// HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
// SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
//
// Source: https://github.com/mmastrac/rust-ctor
#[proc_macro_attribute]
pub fn sly_static(attr: TokenStream, function: TokenStream) -> TokenStream {
	// Parse attribute tokens as a comma-separated list of idents.
	let args = if attr.is_empty() {
		Vec::new()
	} else {
		parse_macro_input!(attr with Punctuated::<Ident, Token![,]>::parse_terminated)
			.into_iter()
			.collect::<Vec<_>>()
	};

	// Allowed values are only "Send" and "Sync".
	let mut traits = Vec::new();
	for arg in args {
		let arg_str = arg.to_string();
		if arg_str != "Send" && arg_str != "Sync" {
			return TokenStream::from(quote! {
				compile_error!("Expected 'Send' and/or 'Sync' as arguments to #[sly_static]");
			});
		}
		traits.push(arg_str);
	}
	// Remove duplicate entries.
	traits.sort();
	traits.dedup();

	let item_static = parse_macro_input!(function as ItemStatic);
	let syn::ItemStatic {
		ident,
		mutability,
		expr,
		attrs,
		ty,
		vis,
		..
	} = item_static;

	if matches!(mutability, syn::StaticMutability::Mut(_)) {
		return TokenStream::from(quote! {
			compile_error!("#[sly_static] static must not be mutable");
		});
	}

	let sly_static_functions_name = sly_static_functions_ident();

	let storage_ident_name = parse_str::<Ident>(&format!("{}_____rust_sly_static_storage", ident))
		.expect("Unable to create storage identifier");

	let initialize_function_name =
		parse_str::<Ident>(&format!("{}_____rust_sly_static_initialize", ident))
			.expect("Unable to create function identifier");

	let trait_impls: Vec<_> = traits
		.into_iter()
		.map(|t| {
			if t == "Sync" {
				quote! {
					unsafe impl ::core::marker::Sync for #ident<#ty> {}
				}
			} else {
				quote! {
					unsafe impl ::core::marker::Send for #ident<#ty> {}
				}
			}
		})
		.collect();

	let expanded = quote! {
		#[allow(non_upper_case_globals)]
		static mut #storage_ident_name: Option<#ty> = None;

		#[doc(hidden)]
		#[allow(non_camel_case_types)]
		#vis struct #ident<T> {
			_data: ::core::marker::PhantomData<T>
		}

		#(#trait_impls)*

		#(#attrs)*
		#vis static #ident: #ident<#ty> = #ident {
			_data: ::core::marker::PhantomData::<#ty>
		};

		impl #ident<#ty> {
			fn set() {
				let val_func = || -> #ty {
					#expr
				};
				let val = Some(val_func());
				unsafe {
					#storage_ident_name = val;
				}
			}
		}

		impl ::core::ops::Deref for #ident<#ty> {
			type Target = #ty;
			fn deref(&self) -> &'static #ty {
				unsafe {
					#storage_ident_name.as_ref().unwrap_unchecked()
				}
			}
		}

		#[used]
		#[allow(non_upper_case_globals)]
		#[sly_static::linkme::distributed_slice(crate::#sly_static_functions_name)]
		#[linkme(crate = sly_static::linkme)]
		static #initialize_function_name: fn() = #ident::set;
	};

	TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn sly_main(_: TokenStream, function: TokenStream) -> TokenStream {
	let input = parse_macro_input!(function as syn::ItemFn);
	let syn::ItemFn {
		attrs,
		block,
		vis,
		sig,
		..
	} = input;

	// if sig.ident != "main" {
	// 	return TokenStream::from(quote! {
	// 		compile_error!("This macro can only be used on the `main` function");
	// 	});
	// }

	let sly_static_functions_name = sly_static_functions_ident();

	(quote! {
		#[sly_static::linkme::distributed_slice]
		#[linkme(crate = sly_static::linkme)]
		pub static #sly_static_functions_name: [fn()];
		#(#attrs)*
		#vis #sig {
			{
				for sly_static_function in #sly_static_functions_name {
					sly_static_function();
				}
			}
			#block
		}
	})
	.into()
}

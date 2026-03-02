use proc_macro::{Delimiter, Group, Punct, Spacing, TokenStream, TokenTree};
use quote::quote;
use syn::{FnArg, ItemFn, Pat, PatIdent, ReturnType, parse_macro_input};

#[proc_macro_attribute]
pub fn prehook(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut attr = attr.into_iter();

    let mut function_sig_stream = Vec::new();
    let function_type = fn_to_fn_ptr(item.clone());
    let function_body;
    let mut dllname = attr
        .next()
        .expect("DLL Name must be specified as a literal.")
        .to_string();
    let _ = attr.next().expect("Must be delimited by a comma.");
    let mut function_name = attr
        .next()
        .expect("Function name must be specified as a literal.")
        .to_string();

    let mut item = item.into_iter();

    loop {
        let token = item.next().unwrap();
        if let TokenTree::Group(ref g) = token {
            if let Delimiter::Brace = g.delimiter() {
                function_body = g.stream();
                break;
            }
        }

        function_sig_stream.push(token);
    }

    let function_args = fn_args_as_params(TokenStream::from_iter(
        function_sig_stream.clone().into_iter(),
    ));

    // Build out our new TokenStream

    // Function headers and function definition

    let mut new_stream = TokenStream::new();
    new_stream.extend::<TokenStream>(
        quote! {
            #[unsafe(no_mangle)]
            extern "system"
        }
        .into(),
    );

    // Thank you quote macro for putting random shit in my strings when being used as a literal
    let _ = dllname.remove(0);
    let _ = dllname.remove(dllname.len() - 1);
    let _ = function_name.remove(0);
    let _ = function_name.remove(function_name.len() - 1);
    new_stream.extend::<TokenStream>(TokenStream::from_iter(
        function_sig_stream.clone().into_iter(),
    ));

    let mut new_body = TokenStream::new();
    new_body.extend::<TokenStream>(
        quote! {
            let c_str = CString::new(#dllname).unwrap();
            let dll_base = LoadLibraryA(c_str.as_ptr() as *const i8);
            let func: extern "system"
        }
        .into(),
    );
    new_body.extend::<TokenStream>(function_type);
    new_body.extend::<TokenStream>(quote!{
        = std::mem::transmute(
            GetProcAddress(dll_base, CString::new(#function_name).unwrap().as_ptr() as *const i8)
        );
    }.into());
    new_body.extend::<TokenStream>(function_body); // Execute prehook code
    new_body.extend::<TokenStream>(
        quote! { // Execute proxy function
            let mut ret = func
        }
        .into(),
    );
    new_body.extend::<TokenStream>(
        TokenTree::Group(Group::new(Delimiter::Parenthesis, function_args)).into(),
    );
    new_body.extend::<TokenStream>(TokenTree::Punct(Punct::new(';', Spacing::Alone)).into());
    new_body.extend::<TokenStream>(
        quote! {
            ret
        }
        .into(),
    );

    let mut unsafe_block = TokenStream::new();
    unsafe_block.extend::<TokenStream>(
        quote! {
            unsafe
        }
        .into(),
    );
    unsafe_block
        .extend::<TokenStream>(TokenTree::Group(Group::new(Delimiter::Brace, new_body)).into());

    new_stream
        .extend::<TokenStream>(TokenTree::Group(Group::new(Delimiter::Brace, unsafe_block)).into());

    new_stream
}

/// If the function has a value being returned, you can modify it with the magic 'ret' variable.
/// This variable has already been defined and made mutable by the macro, so changes can just be made.
#[proc_macro_attribute]
pub fn posthook(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut attr = attr.into_iter();

    let mut function_sig_stream = Vec::new();
    let function_type = fn_to_fn_ptr(item.clone());
    let function_body;
    let mut dllname = attr
        .next()
        .expect("DLL Name must be specified as a literal.")
        .to_string();
    let _ = attr.next().expect("Must be delimited by a comma.");
    let mut function_name = attr
        .next()
        .expect("Function name must be specified as a literal.")
        .to_string();

    let mut item = item.into_iter();

    loop {
        let token = item.next().unwrap();
        if let TokenTree::Group(ref g) = token {
            if let Delimiter::Brace = g.delimiter() {
                function_body = g.stream();
                break;
            }
        }

        function_sig_stream.push(token);
    }

    let function_args = fn_args_as_params(TokenStream::from_iter(
        function_sig_stream.clone().into_iter(),
    ));

    // Build out our new TokenStream

    // Function headers and function definition

    let mut new_stream = TokenStream::new();
    new_stream.extend::<TokenStream>(
        quote! {
            #[unsafe(no_mangle)]
            extern "system"
        }
        .into(),
    );

    // Thank you quote macro for putting random shit in my strings when being used as a literal
    let _ = dllname.remove(0);
    let _ = dllname.remove(dllname.len() - 1);
    let _ = function_name.remove(0);
    let _ = function_name.remove(function_name.len() - 1);
    new_stream.extend::<TokenStream>(TokenStream::from_iter(
        function_sig_stream.clone().into_iter(),
    ));

    let mut new_body = TokenStream::new();
    new_body.extend::<TokenStream>(
        quote! {
            let c_str = CString::new(#dllname).unwrap();
            let dll_base = LoadLibraryA(c_str.as_ptr() as *const i8);
            let func: extern "system"
        }
        .into(),
    );
    new_body.extend::<TokenStream>(function_type);
    new_body.extend::<TokenStream>(quote!{
        = std::mem::transmute(
            GetProcAddress(dll_base, CString::new(#function_name).unwrap().as_ptr() as *const i8)
        );
    }.into());

    new_body.extend::<TokenStream>(
        quote! { // Execute proxy function
            let mut ret = func
        }
        .into(),
    );
    new_body.extend::<TokenStream>(
        TokenTree::Group(Group::new(Delimiter::Parenthesis, function_args)).into(),
    );
    new_body.extend::<TokenStream>(TokenTree::Punct(Punct::new(';', Spacing::Alone)).into());

    new_body.extend::<TokenStream>(function_body); // Execute posthook code

    new_body.extend::<TokenStream>(
        quote! {
            ret
        }
        .into(),
    );

    let mut unsafe_block = TokenStream::new();
    unsafe_block.extend::<TokenStream>(
        quote! {
            unsafe
        }
        .into(),
    );
    unsafe_block
        .extend::<TokenStream>(TokenTree::Group(Group::new(Delimiter::Brace, new_body)).into());

    new_stream
        .extend::<TokenStream>(TokenTree::Group(Group::new(Delimiter::Brace, unsafe_block)).into());

    new_stream
}

/// Allows a fuller control hook of the DLL function.
/// Requires that you call the magic func(...) function, where your hook args
/// are directly passed into it. If this DLL function has a return value,
/// you must capture it and then return it at the end of the hook.
#[proc_macro_attribute]
pub fn fullhook(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut attr = attr.into_iter();

    let mut function_sig_stream = Vec::new();
    let function_type = fn_to_fn_ptr(item.clone());
    let function_body;
    let mut dllname = attr
        .next()
        .expect("DLL Name must be specified as a literal.")
        .to_string();
    let _ = attr.next().expect("Must be delimited by a comma.");
    let mut function_name = attr
        .next()
        .expect("Function name must be specified as a literal.")
        .to_string();

    let mut item = item.into_iter();

    loop {
        let token = item.next().unwrap();
        if let TokenTree::Group(ref g) = token {
            if let Delimiter::Brace = g.delimiter() {
                function_body = g.stream();
                break;
            }
        }

        function_sig_stream.push(token);
    }

    // Build out our new TokenStream

    // Function headers and function definition

    let mut new_stream = TokenStream::new();
    new_stream.extend::<TokenStream>(
        quote! {
            #[unsafe(no_mangle)]
            extern "system"
        }
        .into(),
    );

    // Thank you quote macro for putting random shit in my strings when being used as a literal
    let _ = dllname.remove(0);
    let _ = dllname.remove(dllname.len() - 1);

    let _ = function_name.remove(0);
    let _ = function_name.remove(function_name.len() - 1);
    new_stream.extend::<TokenStream>(TokenStream::from_iter(
        function_sig_stream.clone().into_iter(),
    ));

    let mut new_body = TokenStream::new();
    new_body.extend::<TokenStream>(
        quote! {
            let c_str = CString::new(#dllname).unwrap();
            let dll_base = LoadLibraryA(c_str.as_ptr() as *const i8);
            let func: extern "system"
        }
        .into(),
    );
    new_body.extend::<TokenStream>(function_type);
    new_body.extend::<TokenStream>(quote!{
        = std::mem::transmute(
            GetProcAddress(dll_base, CString::new(#function_name).unwrap().as_ptr() as *const i8)
        );
    }.into());

    new_body.extend::<TokenStream>(function_body); // Execute fullhook code

    let mut unsafe_block = TokenStream::new();
    unsafe_block.extend::<TokenStream>(
        quote! {
            unsafe
        }
        .into(),
    );
    unsafe_block
        .extend::<TokenStream>(TokenTree::Group(Group::new(Delimiter::Brace, new_body)).into());

    new_stream
        .extend::<TokenStream>(TokenTree::Group(Group::new(Delimiter::Brace, unsafe_block)).into());

    new_stream
}

fn fn_to_fn_ptr(input: TokenStream) -> TokenStream {
    // Parse the input as a function definition
    let input_fn = parse_macro_input!(input as ItemFn);

    // Extract parameter types
    let param_types = input_fn
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            match arg {
                FnArg::Typed(pat_type) => Some(&pat_type.ty),
                FnArg::Receiver(_) => None, // Skip self parameters
            }
        })
        .collect::<Vec<_>>();

    // Extract return type
    let return_type = match &input_fn.sig.output {
        ReturnType::Default => quote!(()),
        ReturnType::Type(_, ty) => quote!(#ty),
    };

    // Generate the function pointer type
    let fn_ptr_type = if param_types.is_empty() {
        quote! { fn() -> #return_type }
    } else {
        quote! { fn(#(#param_types),*) -> #return_type }
    };

    // Return the generated TokenStream
    fn_ptr_type.into()
}

fn fn_args_as_params(input: TokenStream) -> TokenStream {
    // Create a dummy function body to make the parser happy
    let mut complete_fn = input.clone();
    complete_fn.extend(TokenStream::from(quote! { {} }));

    // Now parse the completed function
    let input_fn = parse_macro_input!(complete_fn as ItemFn);

    // Extract parameter names
    let param_names = input_fn
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            match arg {
                FnArg::Typed(pat_type) => {
                    // Extract the parameter name from the pattern
                    match &*pat_type.pat {
                        Pat::Ident(PatIdent { ident, .. }) => Some(ident),
                        _ => None, // Skip patterns that aren't simple identifiers
                    }
                }
                FnArg::Receiver(_) => None, // Skip self parameters
            }
        })
        .collect::<Vec<_>>();

    // Generate the parameter list for forwarding
    if param_names.is_empty() {
        return TokenStream::new()
    }
    
    quote! { #(#param_names),* }.into()
    
    
}
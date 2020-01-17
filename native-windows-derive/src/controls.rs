use proc_macro2 as pm2;
use quote::{ToTokens};
use std::cell::RefCell;
use crate::shared::{Parameters, Param};


static TOP_LEVEL: &'static [&'static str] = &[
    "Window", "CanvasWindow", "TabsContainer", "Tab", "MessageWindow"
];

pub struct ControlGen<'a> {
    ty: syn::Ident,
    member: &'a syn::Ident,
    params: RefCell<Parameters>,  
}

impl<'a> ToTokens for ControlGen<'a> {

    fn to_tokens(&self, tokens: &mut pm2::TokenStream) {
        let control_params = self.params.borrow();
        let member = self.member;
        let ty = &self.ty;
    
        let ids: Vec<&syn::Ident> = control_params.params.iter().map(|p| &p.ident).collect();
        let values: Vec<&syn::Expr> = control_params.params.iter().map(|p| &p.e).collect();
    
        let control_tk = quote! {
            nwg::#ty::builder()
                #(.#ids(#values))*
                .build(&mut data.#member)?;
        };

        control_tk.to_tokens(tokens);
    }

}


/// Generate the code that inits the control in the `build_ui` function or the `build_partial` function
/// Note that ordering is done in `organize_controls`
pub fn generate_control<'a>(field: &'a syn::Field) -> Option<ControlGen<'a>> {
    let attrs = &field.attrs;
    if attrs.len() == 0 { return None; }

    let member = field.ident.as_ref().expect("Cannot find member name when generating control");

    let attr = match find_control_attr(&attrs) {
        Some(a) => a,
        None => { return None; }
    };

    let ty = extract_control_type(&field.ty);
    
    let params: Parameters = match syn::parse2(attr.tokens.clone()) {
        Ok(a) => a,
        Err(e) => panic!("Failed to parse field #{}: {}", member, e)
    };

    Some( ControlGen {  ty, member, params: RefCell::new(params)  } )
}

/// Guess the controls parent and reorder the controls creation order to make sure everything works
pub fn organize_controls<'a>(fields: &mut Vec<ControlGen<'a>>) {
    let mut last_top_level: Option<&ControlGen<'a>> = None;

    for control in fields.iter() {
        let flags_name = format!("{}Flags", control.ty);

        if TOP_LEVEL.iter().any(|top| &control.ty == top) {
            last_top_level = Some(control);
            expand_flags_old(control, &flags_name);
            continue;
        }

        let try_expand_parent = match last_top_level.clone() {
            Some(top) => auto_parent(control, top),
            None => true
        };

        if try_expand_parent {
            expand_parent(control);
        }

        expand_flags_old(control, &flags_name);
    }

}

pub fn control_parameters(field: &syn::Field) -> (Vec<syn::Ident>, Vec<syn::Expr>) {
    let member = match field.ident.as_ref() {
        Some(m) => m,
        None => unreachable!()
    };

    let nwg_control = |attr: &&syn::Attribute| {
        attr.path.get_ident()
          .map(|id| id == "nwg_control" )
          .unwrap_or(false)
    };

    let attr = match field.attrs.iter().find(nwg_control) {
        Some(attr) => attr,
        None => unreachable!()
    };

    let ctrl: Parameters = match syn::parse2(attr.tokens.clone()) {
        Ok(a) => a,
        Err(e) => panic!("Failed to parse field #{}: {}", member, e)
    };

    let params = ctrl.params;
    let mut names = Vec::with_capacity(params.len());
    let mut exprs = Vec::with_capacity(params.len());

    for p in params {
        names.push(p.ident);
        exprs.push(p.e);
    }

    (names, exprs)
}

fn find_control_attr(attrs: &[syn::Attribute]) -> Option<&syn::Attribute> {
    let mut index = None;
    for (i, attr) in attrs.iter().enumerate() {
        if let Some(ident) = attr.path.get_ident() {
            if ident == "nwg_control" {
                index = Some(i);
                break;
            }
        }
    }

    index.map(|i| &attrs[i])
}

fn extract_control_type(ty: &syn::Type) -> syn::Ident {
    let control_type: String;

    match ty {
        syn::Type::Path(p) => {
            let path_len = p.path.segments.len();
            control_type = p.path.segments[path_len-1].ident.to_string();
        },
        _ => panic!("Ui control fields must be in a path format `nwg::Button` or simple format `Button`.")
    }

    syn::Ident::new(&control_type, pm2::Span::call_site())
}

/// Expand the control flags from the compressed format. Ex: "WINDOW|VISIBLE"
fn expand_flags_old<'a>(control: &ControlGen<'a>, base: &str) {
    let mut p = control.params.borrow_mut();
    let mut flags = p.params.iter_mut().find(|f| &f.ident == "flags");
    if let Some(flags_param) = flags.as_mut() {
        let flags_value = match &flags_param.e {
            syn::Expr::Lit(expr_lit) => match &expr_lit.lit {
                syn::Lit::Str(value) => value,
                other => panic!("Compressed flags must str, got {:?} for control {}", other, control.member)
            },
            _ => { return; }
        };

        let flags = flags_value.value();
        let splitted: Vec<&str> = flags.split('|').collect();

        let flags_count = splitted.len() - 1;
        let mut final_flags: String = String::with_capacity(100);
        for (i, value) in splitted.into_iter().enumerate() {
            final_flags.push_str("nwg::");
            final_flags.push_str(base);
            final_flags.push_str("::");
            final_flags.push_str(value);

            if i != flags_count {
                final_flags.push('|');
            }
        }

        flags_param.e = match syn::parse_str(&final_flags) {
            Ok(e) => e,
            Err(e) => panic!("Failed to parse flags value for control {}: {}", control.member, e)
        };
    }
}

pub fn expand_flags(member_name: &syn::Ident, ty: &syn::Ident, flags: syn::Expr) -> syn::Expr {
    let flags_type = format!("{}Flags", ty);
    
    let flags_value = match &flags {
        syn::Expr::Lit(expr_lit) => match &expr_lit.lit {
            syn::Lit::Str(value) => value,
            other => panic!("Compressed flags must str, got {:?} for control {}", other, member_name)
        },
        other => panic!("Compressed flags must str, got {:?} for control {}", other, member_name)
    };

    let flags = flags_value.value();
    let splitted: Vec<&str> = flags.split('|').collect();

    let flags_count = splitted.len() - 1;
    let mut final_flags: String = String::with_capacity(100);
    for (i, value) in splitted.into_iter().enumerate() {
        final_flags.push_str("nwg::");
        final_flags.push_str(&flags_type);
        final_flags.push_str("::");
        final_flags.push_str(value);

        if i != flags_count {
            final_flags.push('|');
        }
    }

    match syn::parse_str(&final_flags) {
        Ok(e) => e,
        Err(e) => panic!("Failed to parse flags value for control {}: {}", member_name, e)
    }
}

/// Expand user defined parent field. Ex: "parent: window" becomes "parent: &data.window"
fn expand_parent<'a>(control: &ControlGen<'a>) {
    let mut p = control.params.borrow_mut();
    let mut parent = p.params.iter_mut().find(|f| &f.ident == "parent");

    if let Some(parent_params) = parent.as_mut() {
        let parent_name_path = match &parent_params.e {
            syn::Expr::Path(exp_path) => &exp_path.path.segments,
            _ => { return; }
        };

        let parent_name = match parent_name_path.first().map(|seg| &seg.ident) {
            Some(name) => name,
            None => panic!("Failed to parse parent value for control {}", control.member)
        };

        let final_parent = format!("& data.{}", parent_name);
        parent_params.e = match syn::parse_str(&final_parent) {
            Ok(e) => e,
            Err(e) => panic!("Failed to parse parent value for control {}: {}", control.member, e)
        };
    }
}

/// Add the control parent to the control parameters.
/// Returns `true` if a parent field already exists
/// Returns `false` if the parent field was added
fn auto_parent<'a>(control: &ControlGen<'a>, parent: &ControlGen<'a>) -> bool {
    let mut p = control.params.borrow_mut();
    if p.params.iter().any(|p| &p.ident == "parent") {
        return true;  // User already defined a parent
    }

    let parent_expr = format!("&data.{}", parent.member);
    let parent_param = Param {
        ident: syn::Ident::new("parent", pm2::Span::call_site()),
        sep: syn::token::Colon(pm2::Span::call_site()),
        e: syn::parse_str(&parent_expr).unwrap(),
    };

    p.params.push(parent_param);

    false
}

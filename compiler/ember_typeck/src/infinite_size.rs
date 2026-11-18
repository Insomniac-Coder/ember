//! `[TYP-14]` — `E2200`: a type that contains itself, or holds a view of its own type, with no block of
//! its own between, has no size. Asked once every struct and enum has its fields (`check_infinite_size`),
//! so a field whose type is declared later counts.

use super::*;

impl<'a> Checker<'a> {
    /// Every struct and enum is walked through the fields it holds by value (structs, enums, tuples and
    /// fixed arrays; a counted block, a list, a class, a raw pointer or a function value stops the walk,
    /// since each is a pointer or a block of its own).
    pub(crate) fn check_infinite_size(&mut self) {
        let structs: Vec<StructId> = self.types.structs().map(|(id, _)| id).collect();
        for id in structs {
            let ty = self.types.intern(TyKind::Struct(id));
            let def = self.types.struct_def(id);
            if def.origin.is_some() || self.boxes.contains_key(&id) {
                continue;
            }
            let name = def.name;
            let fields: Vec<(Symbol, Ty, Span)> = def.fields.iter().map(|field| (field.name, field.ty, field.ty_span)).collect();
            if let Some(index) = self.infinite_size_of(ty, name, &fields) {
                // Later passes walk the fields: the refused one is an error type from here on.
                let error = self.common.error;
                self.types.struct_def_mut(id).fields[index].ty = error;
            }
        }
        let enums: Vec<EnumId> = self.types.enums().map(|(id, _)| id).collect();
        for id in enums {
            let ty = self.types.intern(TyKind::Enum(id));
            let def = self.types.enum_def(id);
            if def.name.as_str().starts_with("Option[") || def.name.as_str().starts_with("Result[") {
                continue;
            }
            let name = def.name;
            let fields: Vec<(Symbol, Ty, Span)> = def
                .variants
                .iter()
                .flat_map(|variant| variant.fields.iter().map(|field| (field.name, field.ty, field.ty_span)))
                .collect();
            if let Some(index) = self.infinite_size_of(ty, name, &fields) {
                let error = self.common.error;
                let mut remaining = index;
                for variant in &mut self.types.enum_def_mut(id).variants {
                    if remaining < variant.fields.len() {
                        variant.fields[remaining].ty = error;
                        break;
                    }
                    remaining -= variant.fields.len();
                }
            }
        }
    }

    /// The index of the first field refused, after reporting it.
    fn infinite_size_of(&mut self, own: Ty, name: Symbol, fields: &[(Symbol, Ty, Span)]) -> Option<usize> {
        for (index, &(field, ty, span)) in fields.iter().enumerate() {
            if span == Span::DUMMY {
                continue;
            }
            if self.holds_by_value(ty, own, &mut Vec::new()) {
                let hold = self.with_block(ty, own).map_or_else(String::new, |fixed| {
                    format!(": `{field}: {}`", self.types.display(fixed))
                });
                self.error(
                    codes::E2200,
                    span,
                    format!("`{name}` contains itself through `{field}`, so a `{name}` would have no end; hold the child in its own block{hold}"),
                );
                return Some(index);
            }
            if self.views_own_type(ty, own) {
                self.error(
                    codes::E2200,
                    span,
                    format!(
                        "`{name}.{field}` would be a view of the `{name}` that holds this one, which no struct can keep; for a link back to the parent make `{name}` a class with `children: Array[{name}]` and `{field}: weakref {name} = None`, set with `child.{field} = weakref this` before `this.children.push(owns child)`"
                    ),
                );
                return Some(index);
            }
        }
        None
    }

    /// Whether a value of `ty` holds a `target` inside itself, with no block of its own between.
    fn holds_by_value(&self, ty: Ty, target: Ty, seen: &mut Vec<Ty>) -> bool {
        if ty == target {
            return true;
        }
        if seen.contains(&ty) {
            return false;
        }
        seen.push(ty);
        match self.types.kind(ty) {
            TyKind::Struct(id) if !self.boxes.contains_key(id) => {
                let fields: Vec<Ty> = self.types.struct_def(*id).fields.iter().map(|field| field.ty).collect();
                fields.into_iter().any(|field| self.holds_by_value(field, target, seen))
            }
            TyKind::Enum(id) => {
                let fields: Vec<Ty> = self.types.enum_def(*id).variants.iter().flat_map(|variant| variant.fields.iter().map(|field| field.ty)).collect();
                fields.into_iter().any(|field| self.holds_by_value(field, target, seen))
            }
            TyKind::Tuple(items) => items.clone().into_iter().any(|item| self.holds_by_value(item, target, seen)),
            TyKind::Array { elem, len } if *len > 0 => self.holds_by_value(*elem, target, seen),
            _ => false,
        }
    }

    /// Whether `ty`, held by value, is or holds a view of a `target`.
    fn views_own_type(&self, ty: Ty, target: Ty) -> bool {
        let mut seen = Vec::new();
        self.views_inside(ty, target, &mut seen)
    }

    fn views_inside(&self, ty: Ty, target: Ty, seen: &mut Vec<Ty>) -> bool {
        if seen.contains(&ty) {
            return false;
        }
        seen.push(ty);
        match self.types.kind(ty) {
            TyKind::Ref { inner, .. } => *inner == target,
            TyKind::Span { elem, .. } => *elem == target,
            TyKind::Enum(id) => {
                let fields: Vec<Ty> = self.types.enum_def(*id).variants.iter().flat_map(|variant| variant.fields.iter().map(|field| field.ty)).collect();
                fields.into_iter().any(|field| self.views_inside(field, target, seen))
            }
            TyKind::Tuple(items) => items.clone().into_iter().any(|item| self.views_inside(item, target, seen)),
            TyKind::Array { elem, .. } => self.views_inside(*elem, target, seen),
            _ => false,
        }
    }

    /// `ty` with the `target` it holds by value put in a block of its own (`Tree` to `owns Tree`, `Option[Tree]`
    /// to `Option[owns Tree]`): the line to write. `None` where the holder is not one of those shapes.
    fn with_block(&mut self, ty: Ty, target: Ty) -> Option<Ty> {
        if ty == target {
            return Some(self.box_of(target));
        }
        match self.types.kind(ty).clone() {
            TyKind::Enum(id) if self.is_option(ty) => {
                let inner = self.types.enum_def(id).variants[1].fields[0].ty;
                let fixed = self.with_block(inner, target)?;
                Some(self.option_of(fixed))
            }
            TyKind::Tuple(items) => {
                let mut fixed = Vec::new();
                for item in items {
                    fixed.push(self.with_block(item, target).unwrap_or(item));
                }
                Some(self.types.intern(TyKind::Tuple(fixed)))
            }
            _ => None,
        }
    }
}

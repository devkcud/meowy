use crate::ast::Span;

pub type LocalId = usize;
pub type FunctionId = usize;
pub type BlockId = usize;
pub type EmitId = usize;
pub type CallId = usize;
pub type ReborrowId = usize;
pub type StatementId = usize;
pub type RestartId = usize;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Field {
    pub name: String,
    pub ty: Type,
    pub mutable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    Null,
    Never,
    Bool,
    Int {
        bits: u32,
        signed: bool,
    },
    Float {
        bits: u32,
    },
    String,
    List {
        element: Box<Type>,
        capacity: usize,
    },
    Reference(Box<Type>),
    Record {
        primary: Box<Type>,
        fields: Vec<Field>,
    },
    Union(Vec<Type>),
}

impl Type {
    pub fn is_copy(&self) -> bool {
        match self {
            Self::Null
            | Self::Never
            | Self::Bool
            | Self::Int { .. }
            | Self::Float { .. }
            | Self::String
            | Self::Reference(_) => true,
            Self::List { element, .. } => element.is_copy(),
            Self::Record { primary, fields } => {
                primary.is_copy() && fields.iter().all(|field| field.ty.is_copy())
            }
            Self::Union(types) => types.iter().all(Self::is_copy),
        }
    }

    pub fn layout(&self) -> Option<(usize, usize)> {
        let ty = self;
        match ty {
            Type::Null | Type::Never | Type::Bool => Some((1, 1)),
            Type::Int { bits, .. } | Type::Float { bits } => {
                let size = (*bits as usize).div_ceil(8);
                Some((size, size))
            }
            Type::String => Some((16, 8)),
            Type::Reference(_) => Some((8, 8)),
            Type::Record { primary, fields } => {
                let mut size = 0usize;
                let mut align = 1;
                for ty in
                    std::iter::once(primary.as_ref()).chain(fields.iter().map(|field| &field.ty))
                {
                    let (part, boundary) = ty.layout()?;
                    size = size.checked_next_multiple_of(boundary)?.checked_add(part)?;
                    align = align.max(boundary);
                }
                Some((size.checked_next_multiple_of(align)?, align))
            }
            Type::Union(members) => {
                let mut size = 0;
                for member in members {
                    size = size.max(member.layout()?.0);
                }
                Some((8usize.checked_add(size.checked_next_multiple_of(8)?)?, 8))
            }
            Type::List { element, capacity } => {
                let (size, align) = element.layout()?;
                let start = 8usize.checked_next_multiple_of(align)?;
                let size = start.checked_add(size.checked_mul(*capacity)?)?;
                let align = align.max(8);
                Some((size.checked_next_multiple_of(align)?, align))
            }
        }
    }

    pub fn has_reference(&self) -> bool {
        match self {
            Self::List { element, .. } => element.has_reference(),
            Self::Reference(_) => true,
            Self::Record { primary, fields } => {
                primary.has_reference() || fields.iter().any(|field| field.ty.has_reference())
            }
            Self::Union(types) => types.iter().any(Self::has_reference),
            _ => false,
        }
    }

    pub fn union(types: impl IntoIterator<Item = Type>) -> Self {
        let mut members = Vec::new();
        for ty in types {
            match ty {
                Self::Never => {}
                Self::Union(types) => members.extend(Self::union(types).members().iter().cloned()),
                ty => members.push(ty),
            }
        }
        members.sort();
        members.dedup();
        match members.len() {
            0 => Self::Never,
            1 => members.pop().expect("one union member"),
            _ => Self::Union(members),
        }
    }

    pub fn members(&self) -> &[Self] {
        match self {
            Self::Never => &[],
            Self::Union(types) => types,
            ty => std::slice::from_ref(ty),
        }
    }

    pub fn accepts(&self, value: &Self) -> bool {
        value.members().iter().all(|ty| self.members().contains(ty))
    }

    pub fn intersection(&self, other: &Self) -> Self {
        Self::union(
            self.members()
                .iter()
                .filter(|ty| other.members().contains(ty))
                .cloned(),
        )
    }

    pub fn subtract(&self, other: &Self) -> Self {
        Self::union(
            self.members()
                .iter()
                .filter(|ty| !other.members().contains(ty))
                .cloned(),
        )
    }
}

#[derive(Clone, Debug)]
pub struct Program {
    pub body: Block,
    pub functions: Vec<Function>,
    pub locals: Vec<Type>,
}

#[derive(Clone, Debug)]
pub struct Function {
    pub id: FunctionId,
    pub name: String,
    pub params: Vec<LocalId>,
    pub result: Type,
    pub body: Block,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub id: BlockId,
    pub ty: Type,
    pub stmts: Vec<Stmt>,
}

#[derive(Clone, Debug)]
pub struct IndexStep {
    pub index: Expr,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum WriteStep {
    Field(usize),
    Index(IndexStep),
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Statement {
        id: StatementId,
        stmts: Vec<Stmt>,
    },
    Bind {
        id: LocalId,
        value: Expr,
    },
    SlotAlias {
        id: LocalId,
        target: BlockId,
        field: String,
        mutable: bool,
    },
    Assign {
        id: LocalId,
        value: Expr,
    },
    SetPath {
        id: LocalId,
        path: Vec<WriteStep>,
        value: Expr,
        span: Span,
    },
    Emit {
        id: EmitId,
        target: BlockId,
        field: Option<String>,
        value: Expr,
    },
    If {
        condition: Expr,
        then: Vec<Stmt>,
        otherwise: Vec<Stmt>,
    },
    Leave(BlockId),
    Restart {
        target: BlockId,
        site: RestartId,
    },
    Expr(Expr),
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub ty: Type,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Place {
    pub root: LocalId,
    pub fields: Vec<usize>,
}

#[derive(Clone, Debug)]
pub enum ExprKind {
    Null,
    Bool(bool),
    Int(i128),
    Float(f64),
    String(String),
    List {
        values: Vec<Expr>,
        list: Type,
    },
    ListSize(Box<Expr>),
    ListIndex {
        value: Box<Expr>,
        index: Box<Expr>,
    },
    ListAdd {
        value: Box<Expr>,
        item: Box<Expr>,
    },
    Local(LocalId),
    Borrow(Place),
    TemporaryBorrow {
        id: LocalId,
        statement: StatementId,
        value: Box<Expr>,
    },
    Reborrow {
        site: ReborrowId,
        value: Box<Expr>,
        fields: Vec<usize>,
    },
    ElementBorrow {
        site: ReborrowId,
        value: Box<Expr>,
        index: Box<Expr>,
    },
    Deref(Box<Expr>),
    Unary {
        op: String,
        value: Box<Expr>,
    },
    Binary {
        op: String,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        id: FunctionId,
        site: CallId,
        args: Vec<Expr>,
    },
    Print {
        parts: Vec<Expr>,
        newline: bool,
    },
    Panic {
        parts: Vec<Expr>,
    },
    Block(Block),
    Field {
        value: Box<Expr>,
        index: usize,
    },
    Primary(Box<Expr>),
    StringSize(Box<Expr>),
    Coerce {
        value: Box<Expr>,
    },
    TypeTest {
        value: Box<Expr>,
        ty: Type,
    },
}

#[cfg(test)]
mod tests {
    use super::Type;

    #[test]
    pub(crate) fn unions_are_normalized_sets_with_null_first() {
        let ty = Type::union([
            Type::String,
            Type::Never,
            Type::Null,
            Type::union([Type::Bool, Type::String]),
        ]);
        assert_eq!(ty, Type::Union(vec![Type::Null, Type::Bool, Type::String]));
        assert_eq!(
            Type::union([Type::Never, Type::String, Type::String]),
            Type::String
        );
        assert_eq!(Type::union([]), Type::Never);
        assert!(ty.accepts(&Type::union([Type::String, Type::Null])));
        assert!(!Type::String.accepts(&ty));
        assert_eq!(
            ty.subtract(&Type::Null),
            Type::union([Type::Bool, Type::String])
        );
        assert_eq!(ty.intersection(&Type::String), Type::String);
    }
}

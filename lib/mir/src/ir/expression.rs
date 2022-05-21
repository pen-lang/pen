use super::{
    arithmetic_operation::ArithmeticOperation, byte_string::ByteString, call::Call, case::Case,
    clone_variables::CloneVariables, comparison_operation::ComparisonOperation,
    drop_variables::DropVariables, if_::If, let_::Let, let_recursive::LetRecursive, record::Record,
    record_field::RecordField, try_operation::TryOperation, variable::Variable, variant::Variant,
    BorrowRecordField, DiscardHeap, RetainHeap, ReuseRecord,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    ArithmeticOperation(ArithmeticOperation),
    Boolean(bool),
    BorrowRecordField(BorrowRecordField),
    ByteString(ByteString),
    Call(Call),
    Case(Case),
    CloneVariables(CloneVariables),
    ComparisonOperation(ComparisonOperation),
    DiscardHeap(DiscardHeap),
    DropVariables(DropVariables),
    If(If),
    Let(Let),
    LetRecursive(LetRecursive),
    None,
    Number(f64),
    Record(Record),
    RecordField(RecordField),
    ReuseRecord(ReuseRecord),
    RetainHeap(RetainHeap),
    TryOperation(TryOperation),
    Variable(Variable),
    Variant(Variant),
}

impl From<ArithmeticOperation> for Expression {
    fn from(operation: ArithmeticOperation) -> Self {
        Self::ArithmeticOperation(operation)
    }
}

impl From<bool> for Expression {
    fn from(bool: bool) -> Self {
        Self::Boolean(bool)
    }
}

impl From<BorrowRecordField> for Expression {
    fn from(field: BorrowRecordField) -> Self {
        Self::BorrowRecordField(field)
    }
}

impl From<ComparisonOperation> for Expression {
    fn from(operation: ComparisonOperation) -> Self {
        Self::ComparisonOperation(operation)
    }
}

impl From<CloneVariables> for Expression {
    fn from(clone: CloneVariables) -> Self {
        Self::CloneVariables(clone)
    }
}

impl From<DiscardHeap> for Expression {
    fn from(discard: DiscardHeap) -> Self {
        Self::DiscardHeap(discard)
    }
}

impl From<DropVariables> for Expression {
    fn from(drop: DropVariables) -> Self {
        Self::DropVariables(drop)
    }
}

impl From<Call> for Expression {
    fn from(call: Call) -> Self {
        Self::Call(call)
    }
}

impl From<If> for Expression {
    fn from(if_: If) -> Self {
        Self::If(if_)
    }
}

impl From<LetRecursive> for Expression {
    fn from(let_recursive: LetRecursive) -> Self {
        Self::LetRecursive(let_recursive)
    }
}

impl From<Let> for Expression {
    fn from(let_: Let) -> Self {
        Self::Let(let_)
    }
}

impl From<f64> for Expression {
    fn from(number: f64) -> Self {
        Self::Number(number)
    }
}

impl From<ByteString> for Expression {
    fn from(string: ByteString) -> Self {
        Self::ByteString(string)
    }
}

impl From<Record> for Expression {
    fn from(record: Record) -> Self {
        Self::Record(record)
    }
}

impl From<RecordField> for Expression {
    fn from(field: RecordField) -> Self {
        Self::RecordField(field)
    }
}

impl From<ReuseRecord> for Expression {
    fn from(record: ReuseRecord) -> Self {
        Self::ReuseRecord(record)
    }
}

impl From<RetainHeap> for Expression {
    fn from(drop: RetainHeap) -> Self {
        Self::RetainHeap(drop)
    }
}

impl From<TryOperation> for Expression {
    fn from(operation: TryOperation) -> Self {
        Self::TryOperation(operation)
    }
}

impl From<Variable> for Expression {
    fn from(variable: Variable) -> Self {
        Self::Variable(variable)
    }
}

impl From<Variant> for Expression {
    fn from(variant: Variant) -> Self {
        Self::Variant(variant)
    }
}

impl From<Case> for Expression {
    fn from(case: Case) -> Self {
        Self::Case(case)
    }
}

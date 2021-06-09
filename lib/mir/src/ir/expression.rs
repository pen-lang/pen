use super::{
    arithmetic_operation::ArithmeticOperation, byte_string::ByteString, case::Case,
    clone_variables::CloneVariables, comparison_operation::ComparisonOperation,
    drop_variables::DropVariables, function_application::FunctionApplication, if_::If, let_::Let,
    let_recursive::LetRecursive, record::Record, record_element::RecordElement, variable::Variable,
    variant::Variant,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    ArithmeticOperation(ArithmeticOperation),
    Boolean(bool),
    ByteString(ByteString),
    Case(Case),
    CloneVariables(CloneVariables),
    ComparisonOperation(ComparisonOperation),
    DropVariables(DropVariables),
    FunctionApplication(FunctionApplication),
    If(If),
    Let(Let),
    LetRecursive(LetRecursive),
    Number(f64),
    Record(Record),
    RecordElement(RecordElement),
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

impl From<DropVariables> for Expression {
    fn from(drop: DropVariables) -> Self {
        Self::DropVariables(drop)
    }
}

impl From<FunctionApplication> for Expression {
    fn from(function_application: FunctionApplication) -> Self {
        Self::FunctionApplication(function_application)
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

impl From<RecordElement> for Expression {
    fn from(element: RecordElement) -> Self {
        Self::RecordElement(element)
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

use crate::error::CompileError;
use ast::*;

pub fn compile(number: &Number) -> Result<f64, CompileError> {
    Ok(match number.value() {
        NumberRepresentation::Binary(string) => {
            usize::from_str_radix(string, 2).map_err(|error| CompileError::ParseInteger {
                error,
                position: number.position().clone(),
            })? as f64
        }
        NumberRepresentation::Hexadecimal(string) => {
            usize::from_str_radix(string, 16).map_err(|error| CompileError::ParseInteger {
                error,
                position: number.position().clone(),
            })? as f64
        }
        NumberRepresentation::FloatingPoint(string) => {
            string.parse().map_err(|error| CompileError::ParseFloat {
                error,
                position: number.position().clone(),
            })?
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use position::{Position, test::PositionFake};

    #[test]
    fn compile_binary() {
        for (source, value) in [("0", 0.0), ("1", 1.0), ("01", 1.0), ("10", 2.0)] {
            assert_eq!(
                compile(&Number::new(
                    NumberRepresentation::Binary(source.into()),
                    Position::fake()
                )),
                Ok(value)
            );
        }
    }

    #[test]
    fn compile_hexadecimal() {
        for (source, value) in [
            ("0", 0.0),
            ("1", 1.0),
            ("a", 10.0),
            ("f", 15.0),
            ("01", 1.0),
            ("10", 16.0),
        ] {
            assert_eq!(
                compile(&Number::new(
                    NumberRepresentation::Hexadecimal(source.into()),
                    Position::fake()
                )),
                Ok(value)
            );
        }
    }

    #[test]
    fn compile_decimal_float() {
        for (source, value) in [
            ("0", 0.0),
            ("1", 1.0),
            ("1.0", 1.0),
            ("4.2", 4.2),
            ("-4.2", -4.2),
            ("1e1", 10.0),
            ("1e2", 100.0),
        ] {
            assert_eq!(
                compile(&Number::new(
                    NumberRepresentation::FloatingPoint(source.into()),
                    Position::fake()
                )),
                Ok(value)
            );
        }
    }
}

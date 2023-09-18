#[derive(Clone, Debug)]
pub enum Literal {
    Number(f32),
    String(String),
    Boolean(bool),
    Variable(String),
    Assignment(String, Box<Literal>),
}

impl From<Literal> for String {
    fn from(value: Literal) -> Self {
        match value {
            Literal::String(val) => val,
            Literal::Number(val) => format!("{}", val),
            Literal::Boolean(val) => format!("{}", val),
            Literal::Variable(val) => val,
            Literal::Assignment(name, literal) => {
                let literal = *literal;
                let literal: String = literal.into();

                format!("let {} = {}", name, literal)
            }
        }
    }
}

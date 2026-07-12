#[derive(PartialEq, Debug)]
pub enum TokenKind {
    WhiteSpace,
    LineEnding,
    Comment,
    FieldKey,
    FieldSeparator,
    FieldValue,
}
pub struct Token {
    pub kind: TokenKind,
    pub data: String,
}

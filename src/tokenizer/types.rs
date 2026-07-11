#[derive(PartialEq, Debug)]
pub enum TokenKind {
    WhiteSpace,
    LineEnding,
    Comment,
    Key,
    Separator,
    Value
}
pub struct Token {
    pub kind: TokenKind,
    pub data: String,
}
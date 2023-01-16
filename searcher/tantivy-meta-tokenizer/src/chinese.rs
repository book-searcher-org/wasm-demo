use crate::MetaTokenStream;
use jieba_rs::{Jieba, TokenizeMode};
use tantivy::tokenizer::{BoxTokenStream, Token};

lazy_static::lazy_static! {
    static ref JIEBA: Jieba = jieba_rs::Jieba::new();
}

pub fn token_stream(text: &str) -> BoxTokenStream {
    let text = fast2s::convert(text);

    let mut indices = text.char_indices().collect::<Vec<_>>();
    indices.push((text.len(), '\0'));

    let origin_tokens = JIEBA.tokenize(&text, TokenizeMode::Search, false);
    let origin_tokens_len = origin_tokens.len();

    let all_tokens_len = origin_tokens_len;

    let mut tokens = Vec::with_capacity(all_tokens_len);

    for (token_idx, token) in origin_tokens.iter().enumerate() {
        tokens.push(Token {
            offset_from: indices[token.start].0,
            offset_to: indices[token.end].0,
            text: token.word.to_string(),
            position: token_idx * 2 + 1,
            position_length: all_tokens_len,
        });
    }

    BoxTokenStream::from(MetaTokenStream { tokens, index: 0 })
}

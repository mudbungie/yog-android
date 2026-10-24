//! **A model's markdown, read for the glass** (bl-7355, DESIGN §22 item 4):
//! what a model's prose is made of once its marks are taken off — fenced
//! blocks, `#` headings, and inside the prose the inline spans (code,
//! emphasis, strong). Pure and host-tested; `shell/chat/prose.rs` paints it.
//!
//! **Only the marks models actually write are read**, and a mark that does
//! not close is a literal: these are verbatim server bytes, so `2 * 3`, a
//! stray backtick or an unterminated `**` keep every character. The desktop
//! reads fences the same way (lernie `ui::chat::code`, bl-beef): a line
//! opening with three backticks opens a block, the next closes it, and a
//! fence still open — a streaming turn — runs to the end of the words.

/// The marker a fence line opens with.
const FENCE: &str = "```";

/// One block of what was said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    /// Lines laid as prose, their inline marks read.
    Prose(Vec<Span>),
    /// One `#` heading line, its marks taken off.
    Heading(Vec<Span>),
    /// The lines inside one fence, without it.
    Code(String),
}

/// How one run of prose is set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Plain,
    /// Between backticks: the code face.
    Code,
    /// Between single asterisks. (Underscores are not read: a model's
    /// prose is full of `snake_case`, and a word boundary rule is more
    /// reading than the glass needs.)
    Italic,
    /// Between double asterisks.
    Strong,
}

/// A run of words in one style.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub style: Style,
    pub text: String,
}

/// **Split `words` into blocks.** Empty prose between two blocks is not a
/// block.
#[must_use]
pub fn blocks(words: &str) -> Vec<Block> {
    let mut out = Vec::new();
    let mut held: Vec<&str> = Vec::new();
    let mut fenced = false;
    for line in words.lines() {
        if line.trim_start().starts_with(FENCE) {
            close(&mut out, &mut held, fenced);
            fenced = !fenced;
        } else if let (false, Some(title)) = (fenced, heading(line)) {
            close(&mut out, &mut held, false);
            out.push(Block::Heading(spans(title)));
        } else {
            held.push(line);
        }
    }
    close(&mut out, &mut held, fenced);
    out
}

/// End the block `held` is, if it holds anything worth one.
fn close(out: &mut Vec<Block>, held: &mut Vec<&str>, fenced: bool) {
    let text = held.join("\n");
    held.clear();
    if fenced {
        out.push(Block::Code(text));
    } else if !text.trim().is_empty() {
        out.push(Block::Prose(spans(text.trim_matches('\n'))));
    }
}

/// A heading line's words: one to six `#` and a space, then the title.
fn heading(line: &str) -> Option<&str> {
    let title = line.trim_start_matches('#');
    let marks = line.len() - title.len();
    let title = title.strip_prefix(' ')?;
    (1..=6).contains(&marks).then(|| title.trim())
}

/// **Read the inline marks of one stretch of prose.** Adjacent runs of one
/// style are one span.
#[must_use]
pub fn spans(text: &str) -> Vec<Span> {
    let mut out: Vec<Span> = Vec::new();
    let mut rest = text;
    while !rest.is_empty() {
        let (style, inner, after) = next(rest);
        push(&mut out, style, inner);
        rest = after;
    }
    out
}

/// The next run of `rest`: a marked span if one opens here and closes,
/// else the plain words up to the next character that could open one.
fn next(rest: &str) -> (Style, &str, &str) {
    for style in [Style::Code, Style::Strong, Style::Italic] {
        if let Some((inner, after)) = marked(rest, style) {
            return (style, inner, after);
        }
    }
    // Not a span here: the first character is plain, and so is everything
    // up to the next character that could open one.
    let stop = rest
        .char_indices()
        .skip(1)
        .find(|(_, c)| matches!(c, '`' | '*'))
        .map_or(rest.len(), |(at, _)| at);
    let (words, after) = rest.split_at_checked(stop).unwrap_or((rest, ""));
    (Style::Plain, words, after)
}

/// The words inside `style`'s marks and what follows them, when `rest`
/// opens with the mark and a second one closes it around words that neither
/// start nor end with a space — `2 * 3 * 4` is arithmetic, not emphasis.
/// Code takes any words but none.
fn marked(rest: &str, style: Style) -> Option<(&str, &str)> {
    let mark = match style {
        Style::Code => "`",
        Style::Strong => "**",
        Style::Italic | Style::Plain => "*",
    };
    let (inner, after) = rest.strip_prefix(mark)?.split_once(mark)?;
    let tight = inner.trim() == inner;
    (!inner.is_empty() && (style == Style::Code || tight)).then_some((inner, after))
}

/// Add `text` in `style`, joining the run before it when the styles agree.
fn push(out: &mut Vec<Span>, style: Style, text: &str) {
    match out.last_mut() {
        Some(last) if last.style == style => last.text.push_str(text),
        _ => out.push(Span {
            style,
            text: text.to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests;

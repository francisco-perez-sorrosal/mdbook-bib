# Edge Cases

This chapter demonstrates how edge cases are handled.

## Code Blocks Are Protected

Citations inside code blocks are NOT processed:

```rust
// This @Klabnik2018 won't become a citation
fn main() {
    println!("Hello, world!");  // See @Matsakis2014
}
```

Inline code is also protected: `@Jung2017` stays as-is.

Here's a markdown example:

```markdown
According to @author, this citation syntax is useful.
Use [@key] for parenthetical citations.
```

## Escaped @ Symbols

Use `\@` to produce a literal @ symbol:

- Email: user\@example.com
- Twitter: \@rustlang
- At sign in text: Contact me \@ the conference

## Email Addresses Are Not Matched

Email addresses are automatically excluded:

- support@rust-lang.org
- hello@example.com
- firstname.lastname@company.org

## URLs Are Not Matched

URLs with @ symbols work correctly:

- Git URLs: git@github.com:rust-lang/rust.git
- Mentions in URLs: https://twitter.com/@rustlang

## Mixed Content

This paragraph mixes everything: @Klabnik2018 wrote a great book, available at rust-lang.org.
You can email questions to user\@example.com or check the code in `@config` variables.
The formal proofs [@Jung2017] are also available online.

## Consecutive Citations

Multiple citations can appear close together:

According to @Klabnik2018 and @Blandy2021, Rust is excellent.

See also [@Matsakis2014] and [@Jung2017] for technical details.

## Citations in Lists

- Primary reference: @Klabnik2018
- Academic overview: @Matsakis2014
- Formal verification: @Jung2017
- Community resources: @Ferris2023

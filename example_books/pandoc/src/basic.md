# Basic Citations

This chapter demonstrates the basic Pandoc citation patterns.

## Author-in-Text Citations

Use `@key` when the author is part of the sentence:

> @Klabnik2018 provide an excellent introduction to Rust's ownership system.

> According to @Matsakis2014, Rust achieves memory safety through its type system.

> @Blandy2021 cover advanced topics like async programming and unsafe code.

## Parenthetical Citations

Use `[@key]` for citations in parentheses:

> Rust's ownership model prevents data races at compile time [@Matsakis2014].

> The borrow checker ensures memory safety without runtime overhead [@Klabnik2018].

> For a deeper understanding of Rust's safety guarantees, see the formal proof [@Jung2017].

## Multiple Sentences

You can mix styles naturally in flowing text:

The Rust programming language has gained significant popularity in recent years.
@Klabnik2018 wrote the definitive guide for learning Rust.
The language's novel approach to memory safety has been extensively studied [@Matsakis2014].
More recently, @Jung2017 provided formal proofs of Rust's safety guarantees, which has increased confidence in the language for safety-critical applications.

## Traditional Syntax Still Works

The original mdbook-bib syntaxes continue to work:

- Handlebars style: {{#cite Ferris2023}}
- Double-at shorthand: @@Blandy2021

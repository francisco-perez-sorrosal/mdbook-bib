# Introduction to Alphanumeric Citation Style

This test book demonstrates the **alphanumeric citation style** using mdbook-bib's CSL backend.

## What is Alphanumeric Style?

The alphanumeric style uses **author-based labels** instead of sequential numbers. Citations appear as compact identifiers derived from author names and publication year, like `[Smi24]` or `[JL23]`.

## Expected Behavior

With alphanumeric style, citations should render as:

- Single author: `[Smi24]` (first 3 letters of surname + 2-digit year)
- Two authors: `[JL23]` (initials of both authors + year)
- Multiple authors: `[CB+22]` (first author initial + "+" + year)

## Example Citations

Let's cite some works to see the alphanumeric labels in action:

- A single author work: {{#cite smith2024}}
- A two-author book: {{#cite jones2023}}
- A work with multiple authors: {{#cite chen2022}}

Using shorthand syntax: @@williams2021

The bibliography at the end should show entries with labels like `[Smi24]`, `[JL23]`, `[CB+22]`, and `[Wil21]`.

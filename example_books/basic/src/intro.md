# Basic Test Book

This is a test book for trying stuff with mdbook-bib.

The following snippet is just to demonstrate that we don't override the {{#include}} expression described in issue [#52](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/52):

```rust
{{#include hello.rs}}
```

This is a reference {{#cite mdBook}} that has to be resolved to the right bibliography file.

This is a reference to a non-existing book that reports a bug @@mdBookWithAuthorsWithANDInTheirName that was resolved. See details in the reference. The rendered reference should NOT show the bug anymore.

This is a reference to a bib entry {{#cite DUMMY:1}} which was reported as an issue in [#39](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/39)
This is the same reference to a bib entry as above but with @@DUMMY:1 which was reported as an issue in [#39](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/39)

This is a reference to another bib entry {{#cite 10.1145/3508461}} which was also reported as an issue in [#39](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/39)
Same for the @@10.1145/3508461 version

## License

mdbook-bib is released under the [MPL-2.0 License](https://github.com/francisco-perez-sorrosal/mdbook-bib/blob/master/LICENSE).

#

## [WIP](https://github.com/francisco-perez-sorrosal/mdbook-bib/tree/HEAD)

**Closed issues:**

- Fix alphanumeric CSL style [\#77](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/77)
- Data-Driven CSL Style Resolution [\#74](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/74)
- Config Module Refactoring [\#73](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/73)
- Improve tests [\#70](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/70)
- Automate release project [\#67](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/67)
- Improve default handlebar templates to make cites/references more appealing [\#66](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/66)
- Hayagriva integration [\#62](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/62)

**Merged pull requests:**

- Enhance citation handling with Pandoc compatibility [\#79](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/79) ([francisco-perez-sorrosal](https://github.com/francisco-perez-sorrosal))
- Fix alphanumeric style [\#78](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/78) ([francisco-perez-sorrosal](https://github.com/francisco-perez-sorrosal))
- Enhance CSL backend with new citation styles and improved style loading [\#76](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/76) ([francisco-perez-sorrosal](https://github.com/francisco-perez-sorrosal))
- Refactor template loading in config.rs for improved maintainability [\#75](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/75) ([francisco-perez-sorrosal](https://github.com/francisco-perez-sorrosal))
- Perform test re-organization for improved clarity and maintainability [\#71](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/71) ([francisco-perez-sorrosal](https://github.com/francisco-perez-sorrosal))
- Enhance Makefile and documentation for improved release management [\#69](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/69) ([francisco-perez-sorrosal](https://github.com/francisco-perez-sorrosal))
- Enhance manual and test\_book bibliography rendering and citation syntax [\#68](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/68) ([francisco-perez-sorrosal](https://github.com/francisco-perez-sorrosal))
- Hayagriva integration [\#65](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/65) ([francisco-perez-sorrosal](https://github.com/francisco-perez-sorrosal))

## [v0.5.1](https://github.com/francisco-perez-sorrosal/mdbook-bib/tree/v0.5.1) (2026-01-14)

**Closed issues:**

- Refactor Code to prepare for Hayagriva integration [\#63](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/63)
- This preprocessor overrides mdBook's default built-in `{{\#...}}` preprocessor expressions [\#52](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/52)
- @@ citations with a dot \(.\) at the end, won't render properly [\#49](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/49)
- Extract a single handlebars registry for the different templates of the project [\#47](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/47)

**Merged pull requests:**

- Refactor: Extract modules for improved architecture [\#64](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/64) ([francisco-perez-sorrosal](https://github.com/francisco-perez-sorrosal))
- Single handlebar registry for storing the plugin templates [\#61](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/61) ([francisco-perez-sorrosal](https://github.com/francisco-perez-sorrosal))
- Fix render of @@ citations with a dot \(.\) at the end [\#60](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/60) ([francisco-perez-sorrosal](https://github.com/francisco-perez-sorrosal))
- Fix override of mdBook's default built-in {{\#...}} expressions [\#59](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/59) ([francisco-perez-sorrosal](https://github.com/francisco-perez-sorrosal))
- Add Claude Code GitHub Workflow [\#58](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/58) ([francisco-perez-sorrosal](https://github.com/francisco-perez-sorrosal))

## [v0.5.0](https://github.com/francisco-perez-sorrosal/mdbook-bib/tree/v0.5.0) (2025-12-06)

**Fixed bugs:**

- encountered fatal runtime error: stack overflow when trying to build mdbook [\#39](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/39)

**Closed issues:**

- Switch log for tracing to match mdbook project logging/tracing features [\#57](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/57)
- Build fails with mdbook 0.5.0 [\#53](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/53)
- Interoperability with pandoc [\#51](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/51)
- Make more explicit logging for nom\_bibtex attribute parsing [\#50](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/50)
- Allow to create a bibliography on a page of all citations on that page [\#38](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/38)

**Merged pull requests:**

- replace log with tracing [\#55](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/55) ([tompkins-ct](https://github.com/tompkins-ct))
- \[Breaking / deps\] Update mdbook v0.5.1 [\#54](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/54) ([tompkins-ct](https://github.com/tompkins-ct))

## [v0.0.7](https://github.com/francisco-perez-sorrosal/mdbook-bib/tree/v0.0.7) (2025-07-24)

**Closed issues:**

- Bump nom\_bibtex to 0.5.0 [\#48](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/48)
- Remove duplicate code for finding cite placeholders in text [\#46](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/46)

## [v0.0.6](https://github.com/francisco-perez-sorrosal/mdbook-bib/tree/v0.0.6) (2023-09-02)

**Fixed bugs:**

- Error with overwriting file used to render inline citations. [\#43](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/43)

**Closed issues:**

- Alexander is not Alex and er [\#44](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/44)
- Release v0.0.5 [\#42](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/42)
- Allow References sorting styles [\#32](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/32)

**Merged pull requests:**

- fix: handle author names which include 'and' [\#45](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/45) ([raspofabs](https://github.com/raspofabs))

## [v0.0.5](https://github.com/francisco-perez-sorrosal/mdbook-bib/tree/v0.0.5) (2023-04-13)

**Closed issues:**

- Inline citation styles [\#40](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/40)
- Does not work for nested paths [\#37](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/37)

**Merged pull requests:**

- Citation styles and reference sorting [\#41](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/41) ([mlange-42](https://github.com/mlange-42))

## [v0.0.4](https://github.com/francisco-perez-sorrosal/mdbook-bib/tree/v0.0.4) (2021-09-04)

**Fixed bugs:**

- The @ symbol in the content of a reference is not properly parsed by nom-bibtex [\#25](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/25)
- Citations from sub-directory link to wrong bibliography.html [\#24](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/24)
- Link to bibliography from citations in subfolders [\#34](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/34) ([tchernobog](https://github.com/tchernobog))
- Fix parsing of @ symbol in bibtex content [\#26](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/26) ([francisco-perez-sorrosal](https://github.com/francisco-perez-sorrosal))

**Documentation:**

- Add a Features section in the README.md [\#35](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/35)
- Add and automate a Changelog [\#22](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/22)
- Create a separated user guide describing the plugin configuration options [\#20](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/20)
- Shorten README.md & reference the book for additional info [\#27](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/27) ([francisco-perez-sorrosal](https://github.com/francisco-perez-sorrosal))

**Closed issues:**

- Allow also citations with @ in the same way as yarner-lib [\#31](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/31)
- Release v0.3.0 binaries [\#23](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/23)
- Custom reference styles [\#15](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/15)
- \[Feature\] Make title of bibliography configurable [\#6](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/6)

**Merged pull requests:**

- Allow citations with @@citation-key [\#33](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/33) ([francisco-perez-sorrosal](https://github.com/francisco-perez-sorrosal))
- Fix location of .bib in book scaffold and fix \#24 [\#30](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/30) ([francisco-perez-sorrosal](https://github.com/francisco-perez-sorrosal))
- Allow custom reference styles [\#29](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/29) ([francisco-perez-sorrosal](https://github.com/francisco-perez-sorrosal))
- Add bib title config param [\#28](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/28) ([francisco-perez-sorrosal](https://github.com/francisco-perez-sorrosal))

## [v0.0.3](https://github.com/francisco-perez-sorrosal/mdbook-bib/tree/v0.0.3) (2021-04-04)

**Documentation:**

- Consolidate config style with either underscores or hyphens [\#16](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/16)

**Closed issues:**

- Deploy to releases [\#13](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/13)
- Add more test for the current codebase where necessary [\#12](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/12)
- Extract url from bib entries and render a link on the title of the article [\#8](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/8)
- Allow to copy citations/references to clipboard directly from Bibliography [\#7](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/7)
- \[Feature\] Parse also fields `year` and `month`, if `date` is absent [\#5](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/5)
- \[Feature\] Option to show only cited references [\#2](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/2)
- Collaboration on this project [\#1](https://github.com/francisco-perez-sorrosal/mdbook-bib/issues/1)

**Merged pull requests:**

- \[\#8\] Extract url from bib entries and render a link on the title of t… [\#21](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/21) ([francisco-perez-sorrosal](https://github.com/francisco-perez-sorrosal))
- Consolidate config style with hyphens [\#19](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/19) ([francisco-perez-sorrosal](https://github.com/francisco-perez-sorrosal))
- Parse fields `year` and `month` if `date` is not present [\#18](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/18) ([mlange-42](https://github.com/mlange-42))
- More tests for critical functionality [\#17](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/17) ([francisco-perez-sorrosal](https://github.com/francisco-perez-sorrosal))
- Publish to Releases [\#14](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/14) ([mlange-42](https://github.com/mlange-42))
- Parse toml config table into struct [\#11](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/11) ([mlange-42](https://github.com/mlange-42))
- Allow to copy citations to clipboard from Bibliography [\#10](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/10) ([francisco-perez-sorrosal](https://github.com/francisco-perez-sorrosal))
- List cited refs only [\#9](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/9) ([mlange-42](https://github.com/mlange-42))
- Linters: rustfmt + clippy in CI, fix all warnings, cleanup dependencies [\#3](https://github.com/francisco-perez-sorrosal/mdbook-bib/pull/3) ([mlange-42](https://github.com/mlange-42))

## [v0.0.2](https://github.com/francisco-perez-sorrosal/mdbook-bib/tree/v0.0.2) (2021-02-14)

## [v0.0.1](https://github.com/francisco-perez-sorrosal/mdbook-bib/tree/v0.0.1) (2021-02-10)



\* *This Changelog was automatically generated by [github_changelog_generator](https://github.com/github-changelog-generator/github-changelog-generator)*

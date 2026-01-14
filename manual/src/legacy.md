# Legacy Backend (Handlebars)

The Legacy backend uses [Handlebars](https://handlebarsjs.com/) templates to render citations and bibliography entries. This gives you full control over the HTML output, including custom layouts, interactive elements, and styling.

## When to Use Legacy Backend

- You need custom HTML layouts for bibliography entries
- You want interactive elements (copy buttons, collapsible details, tooltips)
- You need to match a specific visual style not available in standard CSL formats
- You want to embed custom JavaScript functionality

## Configuration

The Legacy backend is the default. No `backend` option is needed:

```toml
[preprocessor.bib]
bibliography = "refs.bib"

# Optional customization
hb-tpl = "render/references.hbs"      # Bibliography entry template
cite-hb-tpl = "render/citation.hbs"   # Inline citation template
css = "render/style.css"              # Custom CSS
js = "render/script.js"               # Custom JavaScript
```

## Template Variables

### Bibliography Entry Template (`hb-tpl`)

Available variables for rendering each bibliography entry:

| Variable | Type | Description |
|----------|------|-------------|
| `citation_key` | String | Unique identifier for the entry |
| `title` | String | Entry title |
| `authors` | Array | List of authors, each as `[Last, First]` |
| `pub_year` | String | Publication year |
| `pub_month` | String | Publication month |
| `url` | String | URL if available |
| `summary` | String | Abstract/summary |
| `index` | Number | Citation order (1-based) |
| `entry_type` | String | Type: article, book, inproceedings, etc. |
| `doi` | String | DOI if available |
| `pages` | String | Page numbers |
| `volume` | String | Volume number |
| `issue` | String | Issue number |
| `publisher` | String | Publisher name |
| `address` | String | Publisher location |
| `editor` | Array | Editors (same format as authors) |
| `edition` | String | Edition |
| `series` | String | Series name |
| `note` | String | Additional notes |

### Citation Template (`cite-hb-tpl`)

Available variables for inline citations:

| Variable | Type | Description |
|----------|------|-------------|
| `path` | String | Relative path to bibliography page |
| `item.citation_key` | String | Citation key |
| `item.title` | String | Entry title |
| `item.authors` | Array | Authors |
| `item.pub_year` | String | Year |
| `item.index` | Number | Citation order |
| (all other `item.*` fields) | | Same as bibliography template |

## Example Templates

### Simple Bibliography Entry

```handlebars
<div class="bib-entry" id="{{citation_key}}">
  <span class="bib-index">[{{index}}]</span>
  <span class="bib-authors">
    {{#each authors}}{{#unless @first}}, {{/unless}}{{this.[1]}} {{this.[0]}}{{/each}}
  </span>
  <span class="bib-title">"{{title}}"</span>
  {{#if pub_year}}<span class="bib-year">({{pub_year}})</span>{{/if}}
  {{#if url}}<a href="{{url}}">[link]</a>{{/if}}
</div>
```

### Simple Citation

```handlebars
<a href="{{path}}#{{item.citation_key}}">[{{item.index}}]</a>
```

### Citation with Hover Preview

```handlebars
<a href="{{path}}#{{item.citation_key}}"
   class="citation"
   title="{{item.title}} ({{item.pub_year}})">
  [{{item.index}}]
</a>
```

### Bibliography Entry with Copy Button

```handlebars
<div class="bib-entry" id="{{citation_key}}">
  <div class="bib-header">
    <span class="bib-index">[{{index}}]</span>
    <button class="copy-btn" onclick="copyBib('{{citation_key}}')">Copy</button>
  </div>
  <div class="bib-content">
    <strong>{{title}}</strong><br>
    {{#each authors}}{{this.[1]}} {{this.[0]}}{{#unless @last}}, {{/unless}}{{/each}}
    {{#if pub_year}}({{pub_year}}){{/if}}
  </div>
</div>
```

## Custom CSS

Style your bibliography with CSS:

```css
.bib-entry {
  margin: 1em 0;
  padding: 0.5em;
  border-left: 3px solid #4a9eff;
}

.bib-index {
  font-weight: bold;
  color: #4a9eff;
}

.bib-title {
  font-style: italic;
}

.citation {
  color: #4a9eff;
  text-decoration: none;
}

.citation:hover {
  text-decoration: underline;
}
```

## Custom JavaScript

Add interactivity with JavaScript:

```javascript
function copyBib(key) {
  const entry = document.getElementById(key);
  const text = entry.querySelector('.bib-content').innerText;
  navigator.clipboard.writeText(text);
}
```

## Full Configuration Example

```toml
[preprocessor.bib]
bibliography = "references.bib"
title = "References"
render-bib = "cited"          # Only show cited entries
order = "index"               # Order by citation appearance
hb-tpl = "render/refs.hbs"
cite-hb-tpl = "render/cite.hbs"
css = "render/bib.css"
js = "render/bib.js"
```

## Tips

- Use `{{#if field}}...{{/if}}` to conditionally render optional fields
- Use `{{#each authors}}...{{/each}}` to iterate over author lists
- Access array elements with `{{this.[0]}}` (last name) and `{{this.[1]}}` (first name)
- Add `id="{{citation_key}}"` to entries for citation linking
- Use the `index` field for numbered citations

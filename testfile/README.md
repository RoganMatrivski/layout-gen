# XML Layout API Definition

This document defines an XML-based API for describing UI layouts that can be ingested by the `layout-gen` engine.

## Supported Elements
The XML API defines the following core elements:
- `<layout>`: The root container.
- `<flex>`: A flexbox layout container.
- `<block>`: A rectangular block element.
- `<grid>`: A grid layout container.
- `<draw>`: A drawable particle nested within other elements.

## Attribute Reference

### Common Attributes
- `id`: Optional unique identifier.
- `width`, `height`, `min-width`, `min-height`, `max-width`, `max-height`: Opaque strings (e.g., `px`, `%`).
- `padding`, `margin`: Opaque strings.

### Flex Attributes
- `direction`: `row`, `column`
- `reverse`: `true`, `false`
- `wrap`: `no-wrap`, `wrap`, `reverse-wrap`
- `justify-content`: `start`, `end`, `center`, `space-between`, `space-around`, `space-evenly`
- `align-items`, `align-content`, `align-self`: `start`, `end`, `center`, `stretch`
- `gap-row`, `gap-column`: decimal values.
- `grow`, `shrink`: decimal values.
- `basis`: opaque string.

### Grid Attributes
- `columns`, `rows`: Opaque strings (e.g., `repeat(4,1fr)`).
- `gap-row`, `gap-column`, `flex-grow`, `flex-shrink`: decimal values.

### Draw Attributes
- `component`: Required string.
- `variant`: Required string.
- `size`: `sm`, `md`, `lg`, `xl` (default `md`).
- `align`: `top-left`, `top-center`, `top-right`, `center-left`, `center`, `center-right`, `bottom-left`, `bottom-center`, `bottom-right`.
- `fit`: `fill`, `contain`, `cover`, `none`, `scale-down`.
- `overflow`: `visible`, `hidden`.
- `opacity`: decimal `0.0`–`1.0`.

## Example Layout

```xml
<layout id="app">
  <flex direction="row" grow="1">
    <block width="200px" id="sidebar"/>
    <block id="main">
      <draw component="image" variant="hero" size="lg" align="center"/>
    </block>
  </flex>
</layout>
```

## Implementation Notes
- The engine uses `taffy` as the underlying layout calculator.
- All styles are parsed into the Taffy `Style` struct.
- The `collect_rects` function generates `RenderRect` objects which can be used to render the final layout calculated from this XML structure.

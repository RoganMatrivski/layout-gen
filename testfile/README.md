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
- `width`, `height`, `min-width`, `min-height`, `max-width`, `max-height`: Opaque strings (e.g., `200px`, `50%`, `auto`), each set independently — there is no combined `size` shorthand.
- `padding`, `margin`: Opaque strings, support CSS-style 1/2/4-value box shorthand (e.g. `"12px"`, `"8px 16px"`).
- `flex-grow` (default `0.0`), `flex-shrink` (default `1.0`): decimal values, available on `<block>` and `<grid>`. (`<flex>` uses `grow`/`shrink` instead — see below.)

### Flex Attributes
- `direction`: `row` (default), `column`
- `reverse`: `true`, `false` (default `false`)
- `wrap`: `no-wrap` (default), `wrap`, `reverse-wrap`
- `justify-content`: `start` (default), `end`, `center`, `space-between`, `space-around`, `space-evenly`
- `align-items` (default `stretch`), `align-content` (default `start`), `align-self` (unset by default): `start`, `end`, `center`, `stretch`
- `gap-row`, `gap-column`: decimal values, default `0.0`.
- `grow` (default `0.0`), `shrink` (default `1.0`): decimal values. Note these items don't auto-grow to fill space; opt in explicitly with `grow="1"` where needed.
- `basis`: opaque string, default `auto`.

### Grid Attributes
- `columns`, `rows`: Opaque strings (e.g., `repeat(4,1fr)`, `100px 1fr auto`), default `none`.
- `gap-row`, `gap-column`: decimal values, default `0.0`.
- `flex-grow` (default `0.0`), `flex-shrink` (default `1.0`): decimal values — a `<grid>` is itself a flex item within its parent, same as `<block>`.

### Draw Attributes
- `component`: Required string — the drawable's component type (e.g. `image`, `text`, `icon`, `chart`, `rect`).
- `variant`: Required string — a style/variant token within that component (e.g. `heading`, `ghost`, `brand-mark`), typically resolved by the renderer's own theme/design-system lookup.
- `size`: `sm`, `md`, `lg`, `xl` (default `md`) — the drawable's own intrinsic/preferred size. Distinct from `width`/`height` on its parent element.
- `align`: `top-left`, `top-center`, `top-right`, `center-left`, `center` (default), `center-right`, `bottom-left`, `bottom-center`, `bottom-right` — where within the parent's rect to position the drawable.
- `fit`: `fill` (default), `contain`, `cover`, `none`, `scale-down` — how to reconcile the drawable's intrinsic `size` against the parent rect's actual dimensions (same semantics as CSS `object-fit`).
- `overflow`: `visible` (default), `hidden` — whether content exceeding the parent rect is clipped.
- `opacity`: decimal `0.0`–`1.0`, default `1.0`.

### `<draw>` Semantics — Important

`<draw>` is **not a layout element**. Unlike `<flex>`/`<block>`/`<grid>`, a `<draw>` never becomes its own node in the Taffy tree and never affects sibling/parent sizing, gaps, or flex distribution. It is parsed as metadata attached to its **parent** element and carried through to that parent's `RenderRect` (see Implementation Notes).

Rules:
- A `<draw>` may appear as a child of any `<flex>`, `<block>`, or `<grid>` element.
- At most **one** `<draw>` is permitted per parent element. A parent with zero `<draw>` children simply has no drawable content (`RenderRect.draw` is `None`).
- `<draw>` may coexist with ordinary layout children (`<flex>`/`<block>`/`<grid>`) under the same parent — the parent both lays out its child elements *and* carries its own drawable, independently.

```xml
<block width="200px" height="32px" id="header-logo">
  <draw component="image" variant="brand-mark" size="md" align="center" fit="contain"/>
</block>
```

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
- Each `RenderRect` carries a `draw: Option<DrawProperties>` field, populated from that leaf's `<draw>` child (if any). A renderer should treat `RenderRect` as the single source of truth for both **where** to paint (`x`/`y`/`width`/`height`) and **what** to paint (`draw`); no separate lookup against the Taffy tree or XML source should be necessary.
- A `RenderRect` with `draw: None` represents a pure layout container (no visual content of its own) and can typically be skipped by a renderer that only paints drawable content — though it may still be useful for background/border rendering on containers in the future.

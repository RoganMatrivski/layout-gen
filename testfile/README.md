# XML Layout API Definition

This document defines an XML-based API for describing UI layouts that can be ingested by the `layout-gen` engine.

## Overview

The XML API provides a structured way to define tree-based UI layouts, mapping to Taffy styles and structures.

## Schema Structure

Each layout is defined within a `<layout>` root element.

### Element: `<node>`
Represents a UI element.

- **Attributes:**
    - `id`: Optional unique identifier for the node.
    - `label`: Optional display name for debugging/rendering.

- **Child Elements:**
    - `<style>`: Defines the Taffy style properties.
    - `<node>` (nested): Represents children of the current node.

## Full Style Attribute Reference

The following Taffy-supported properties can be used within the `<style>` block:

### Display & Layout
- `display`: `none`, `flex`, `grid`
- `position`: `relative`, `absolute`
- `flex_direction`: `row`, `column`, `row-reverse`, `column-reverse`
- `flex_wrap`: `no-wrap`, `wrap`, `wrap-reverse`
- `flex_grow`: `float` (e.g., `1.0`)
- `flex_shrink`: `float` (e.g., `1.0`)
- `flex_basis`: `auto`, `px`, `%`

### Sizing
- `width`: `auto`, `px`, `%`
- `height`: `auto`, `px`, `%`
- `min_width`, `max_width`: `auto`, `px`, `%`
- `min_height`, `max_height`: `auto`, `px`, `%`
- `aspect_ratio`: `float`

### Alignment & Spacing
- `justify_content`: `flex-start`, `flex-end`, `center`, `space-between`, `space-around`, `space-evenly`
- `align_items`: `flex-start`, `flex-end`, `center`, `baseline`, `stretch`
- `align_self`: `auto`, `flex-start`, `flex-end`, `center`, `baseline`, `stretch`
- `align_content`: `flex-start`, `flex-end`, `center`, `space-between`, `space-around`, `stretch`
- `gap`: `px`, `%`
- `margin`, `padding`: `px`, `%` (can be specified as `top`, `bottom`, `left`, `right`)
- `inset`: `px`, `%` (for absolute positioning)

## Example Layout

```xml
<layout>
  <node id="root" label="AppContainer">
    <style>
      <display>flex</display>
      <width>100%</width>
      <height>100%</height>
    </style>
    <node id="header" label="Header">
      <style>
        <height>50px</height>
        <background_color>#f0f0f0</background_color>
      </style>
    </node>
    <node id="content" label="MainContent">
      <style>
        <flex_grow>1</flex_grow>
      </style>
    </node>
  </node>
</layout>
```

## Implementation Notes
- The engine uses `taffy` as the underlying layout calculator.
- All styles are parsed into the Taffy `Style` struct.
- The `collect_rects` function generates `RenderRect` objects which can be used to render the final layout calculated from this XML structure.

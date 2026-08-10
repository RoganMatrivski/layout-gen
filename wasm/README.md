# @robinmauritz/layout-gen-wasm

WebAssembly bindings for `layout-gen` - layout calculation and drawable rect generation powered by Taffy and Rust.

## Installation

```bash
npm install @robinmauritz/layout-gen-wasm
```

## Usage

```javascript
import { get_drawable_rects } from '@robinmauritz/layout-gen-wasm';

const xmlLayout = '<layout><flex grow="1"><draw type="line"/></flex></layout>';
const width = 100;
const height = 100;

try {
  const rects = get_drawable_rects(xmlLayout, width, height);
  console.log('Drawable rects:', rects);
} catch (error) {
  console.error('Failed to compute layout:', error);
}
```

## API

### `get_drawable_rects(xml: string, width: number, height: number): RenderRect[]`

Parses an XML layout description, computes layout geometry using Taffy, and returns an array of computed `RenderRect` objects.

- **`xml`**: The layout specification XML string.
- **`width`**: Available width constraint (px).
- **`height`**: Available height constraint (px).
- **Returns**: `RenderRect[]` representing computed drawable elements.

## License

Dual-licensed under either Apache License 2.0 or MIT License.

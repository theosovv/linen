# Fragment

`Fragment` creates a [DocumentFragment](https://developer.mozilla.org/en-US/docs/Web/API/DocumentFragment) from the provided children.  
This is useful for grouping multiple elements without adding extra nodes to the DOM.

## Usage

```tsx
import { Fragment } from 'linen';

const frag = (
  <Fragment>
    <span>A</span>
    <b>B</b>
  </Fragment>
);
// frag is a DocumentFragment containing two child elements
```

## Parameters

- `children`: A single node or an array of nodes to include in the fragment.

## Returns

- A `DocumentFragment` containing the provided children.

## Example

```tsx
const fragment = (
  <Fragment>
    <li>Item 1</li>
    <li>Item 2</li>
  </Fragment>
);
```
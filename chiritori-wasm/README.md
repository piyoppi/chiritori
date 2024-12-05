# WebAssembly of Chiritori

[Chiritori](https://github.com/piyoppi/chiritori) WebAssembly build (This library is experimental).

## Usage

```typescript
import init, { list_all, clean } from "@piyoppi/chiritori-wasm";

await init();

  const initialContent = `
<!DOCTYPE html>
<html>
  <head>
    <title>Sample Code</title>
    <link rel="stylesheet" type="text/css" href="style.css">
  </head>
  <body>
    <h1>Hello World</h1>
    <p>This is a sample page with some code.</p>
    <!-- <time-limited to="2019-12-31 23:59:59"> -->
      <p>This content is only available until the end of the year.</p>
      <p>After that, it will be removed from the page.</p>
    <!-- </time-limited> -->
    <!-- <time-limited to="2999-12-31 23:59:59"> -->
      Campaign!
    <!-- </time-limited> -->
    <!-- <removal-marker name="awesome-campaign"> -->
      40% off! Only until the 2001/12/31!
    <!-- </removal-marker> -->
  </body>
</html>
`;

const configuration = {
  time_limited_configuration: {
    tag_name: "time-limited",
    time_offset: "+00:00",
  },
  removal_marker_configuration: {
    tag_name: "removal-marker",
    targets: ["awesome-campaign"],
  }
};

const delimiterStart = "<!-- <";
const delimiterEnd = "> -->";

// List all removal targets
console.log(list_all(content.value, delimiterStart, delimiterEnd, configuration));

// Remove all targets
console.log(clean(content.value, delimiterStart, delimiterEnd, configuration));
```

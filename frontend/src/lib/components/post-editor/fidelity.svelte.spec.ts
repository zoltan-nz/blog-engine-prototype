import { afterEach, describe, expect, it } from "vitest";
import { Crepe } from "@milkdown/crepe";
import "@milkdown/crepe/theme/common/style.css";
import "@milkdown/crepe/theme/classic.css";

// Confirmed risk (see spec section 5, "Known risk"): remark-stringify
// normalizes formatting even in untouched regions on Crepe 7.22.1— `-` list
// markers become `*`. (7.21.2 also inserted blank lines between list items.)
// Accepted trade-off: posts get reformatted (not mangled — content and
// structure are preserved) the first time they're opened and saved. This
// test pins the exact normalized shape so a Crepe/remark upgrade that
// changes it further gets caught, not silently shipped.
const FIXTURE = `# Heading

Some **bold** text and a [link](https://example.com).

- one
- two
- three

\`\`\`js
function hello() {
  return "world";
}
\`\`\`
`;

const EXPECTED_AFTER_NORMALIZATION = `# Heading

Some **bold** text and a [link](https://example.com).

* one
* two
* three

\`\`\`js
function hello() {
  return "world";
}
\`\`\``;

describe("Crepe markdown round-trip", () => {
  let crepe: Crepe | null = null;

  afterEach(async () => {
    await crepe?.destroy();
    crepe = null;
  });

  it("preserves headings, bold, links, and code fences; normalizes list markers/spacing", async () => {
    const root = document.createElement("div");
    document.body.appendChild(root);

    crepe = new Crepe({ root, defaultValue: FIXTURE });
    await crepe.create();

    expect(crepe.getMarkdown().trim()).toBe(
      EXPECTED_AFTER_NORMALIZATION.trim(),
    );

    root.remove();
  });
});

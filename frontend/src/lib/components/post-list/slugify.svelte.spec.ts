import { describe, expect, it } from "vitest";
import { slugify } from "./slugify.js";

describe("slugify", () => {
  it("lowercases and hyphenates spaces", () => {
    expect(slugify("Hello World!")).toBe("hello-world");
  });

  it("collapses runs of punctuation into one hyphen", () => {
    expect(slugify("A -- B  ..  C")).toBe("a-b-c");
  });

  it("strips leading and trailing hyphens", () => {
    expect(slugify("  --Hello--  ")).toBe("hello");
  });

  it("truncates to 64 characters", () => {
    const long = "a".repeat(100);
    expect(slugify(long)).toHaveLength(64);
  });
});

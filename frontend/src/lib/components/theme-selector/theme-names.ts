const themeNames = [
  "catppuccin",
  "cerberus",
  "concord",
  "crimson",
  "fennec",
  "hamlindigo",
  "legacy",
  "mint",
  "modern",
  "mona",
  "nosh",
  "nouveau",
  "pine",
  "reign",
  "rocket",
  "rose",
  "sahara",
  "seafoam",
  "terminus",
  "vintage",
  "vox",
  "wintry",
] as const;

type ThemeName = (typeof themeNames)[number];

// Maps each theme to the primary font family it prefers.
// The value matches a Font.family key in font-names.ts.
// Themes with empty --base-font-family (catppuccin, modern) default to Inter.
const themeFontFamilyMap: Record<ThemeName, string> = {
  catppuccin: "Inter",
  cerberus: "Inter",
  concord: "Inter",
  crimson: "Montserrat",
  fennec: "Barlow Condensed",
  hamlindigo: "Ubuntu",
  legacy: "Inter",
  mint: "Inter",
  modern: "Inter",
  mona: "Inter",
  nosh: "Montserrat",
  nouveau: "Inter",
  pine: "Bitter",
  reign: "Ubuntu",
  rocket: "Inter",
  rose: "Ubuntu",
  sahara: "Ubuntu",
  seafoam: "Inter",
  terminus: "Montserrat",
  vintage: "Montserrat",
  vox: "Ubuntu",
  wintry: "Inter",
};

export type { ThemeName };
export { themeNames, themeFontFamilyMap };

import { fonts, type Font } from "../components/font-selector/font-names.ts";

function loadGoogleFont(url: string): void {
  if (document.querySelector(`link[href="${url}"]`)) return;
  const link = document.createElement("link");
  link.rel = "stylesheet";
  link.href = url;
  document.head.appendChild(link);
}

class FontStore {
  family = $state(localStorage.getItem("font-family") ?? "Inter");

  apply(font: Font): void {
    if (font.googleParam) {
      const url = `https://fonts.googleapis.com/css2?family=${font.googleParam}&display=swap`;
      loadGoogleFont(url);
      localStorage.setItem("font-url", url);
    } else {
      localStorage.removeItem("font-url");
    }
    document.documentElement.style.setProperty("--font-family", font.cssStack);
    const ls = font.letterSpacing ?? "0";
    document.documentElement.style.setProperty("--letter-spacing", ls);
    localStorage.setItem("font-family", font.family);
    localStorage.setItem("font-css-stack", font.cssStack);
    localStorage.setItem("font-letter-spacing", ls);
    this.family = font.family;
  }

  applyByFamily(family: string): void {
    const font = fonts.find((f) => f.family === family);
    if (font) this.apply(font);
  }
}

export const fontStore = new FontStore();
